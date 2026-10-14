//! Colaboração em tempo real (Nova Live) no motor do Nova Light.
//!
//! O que se compartilha é **uma foto e a revelação dela**, nunca a biblioteca pessoal:
//! - Quem entra recebe a foto original uma vez (ou a mesma cena de demonstração) e a revelação.
//! - Comandos que mudam a revelação da foto compartilhada viram [`LiveChange::Op`]; os ids da foto
//!   nos parâmetros são traduzidos para o id local de cada pessoa.
//! - Desfazer, refazer e gestos mandam só a revelação ([`Session::live_replace_snapshot`]).
//! - Notas, marcações, álbuns e as outras fotos de cada um não saem do computador dele.

use std::sync::Arc;

use base64::Engine as _;
use novalight_catalog::{PhotoId, Source};
use novalight_develop::DevelopSettings;
use serde_json::{Value, json};

use crate::{EngineError, Result, Session};

#[derive(Clone, Debug, PartialEq)]
pub enum LiveChange {
    Op { cmd: String, params: Value, ctx: Value },
    Replace,
}

#[derive(Default)]
pub struct LiveState {
    /// A foto compartilhada, com o id desta biblioteca.
    pub photo: Option<PhotoId>,
    pub outbox: Vec<LiveChange>,
    pub(crate) applying: bool,
    /// Revelação já informada à sessão (muda de objeto a cada alteração).
    pub(crate) seen: Option<Arc<DevelopSettings>>,
}

fn err(msg: impl std::fmt::Display) -> EngineError {
    EngineError::BadParams { cmd: "colaboração".into(), msg: msg.to_string() }
}

/// Impressão digital estável (FNV-1a 64).
pub fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Troca o id da foto de quem editou pelo id local, onde os comandos nomeiam fotos.
fn translate(params: &Value, from: u64, to: u64) -> Value {
    match params {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| {
                    let v = if matches!(k.as_str(), "photo" | "photos" | "id" | "ids" | "target" | "targets") {
                        swap(v, from, to)
                    } else {
                        translate(v, from, to)
                    };
                    (k.clone(), v)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(|v| translate(v, from, to)).collect()),
        other => other.clone(),
    }
}

fn swap(v: &Value, from: u64, to: u64) -> Value {
    match v {
        Value::Number(n) if n.as_u64() == Some(from) => json!(to),
        Value::Array(items) => Value::Array(items.iter().map(|x| swap(x, from, to)).collect()),
        other => other.clone(),
    }
}

// Only audited document edits may execute on a peer. Other local edits travel as data.
fn remote_command_allowed(cmd: &str) -> bool {
    matches!(cmd, "develop.adjust")
}

impl Session {
    fn shared(&self) -> Result<PhotoId> {
        let id = self.live.photo.ok_or_else(|| err("nenhuma foto compartilhada"))?;
        self.catalog.photo(id).map(|_| id).ok_or_else(|| err("a foto compartilhada saiu da biblioteca"))
    }

    /// Compartilha a foto ativa.
    pub fn live_start(&mut self) -> Result<()> {
        let id = self.active().ok_or_else(|| err("abra uma foto para colaborar"))?;
        self.live = LiveState { photo: Some(id), seen: self.develop_of(id), ..LiveState::default() };
        Ok(())
    }

    /// Convidado: ainda sem foto; ela chega com o primeiro documento.
    pub fn live_start_empty(&mut self) {
        self.live = LiveState::default();
    }

    pub fn live_stop(&mut self) {
        self.live = LiveState::default();
    }

    pub fn live_active(&self) -> bool {
        self.live.photo.is_some()
    }

    pub fn live_take(&mut self) -> Vec<LiveChange> {
        if let Ok(id) = self.shared() {
            let now = self.develop_of(id);
            if !ptr_eq(&now, &self.live.seen) {
                self.live.seen = now;
                self.live.outbox.push(LiveChange::Replace);
            }
        }
        std::mem::take(&mut self.live.outbox)
    }

    /// Arrastando um controle: a revelação inteira espera o gesto terminar.
    pub fn live_busy(&self) -> bool {
        self.interaction.is_some()
    }

    /// Foto + revelação (para quem entra).
    pub fn live_snapshot(&self) -> Result<Vec<u8>> {
        let id = self.shared()?;
        let p = self.catalog.photo(id).ok_or_else(|| err("sem foto"))?;
        let mut v = json!({ "name": p.file_name, "develop": &*p.develop });
        match &p.source {
            Source::Demo { scene } => v["demo"] = json!(scene),
            Source::File { path } => {
                let bytes = std::fs::read(path).map_err(|e| err(format!("não consegui ler a foto original: {e}")))?;
                v["hash"] = json!(fnv64(&bytes));
                v["file"] = json!(base64::engine::general_purpose::STANDARD.encode(bytes));
            }
        }
        serde_json::to_vec(&v).map_err(err)
    }

    /// Só a revelação (depois de desfazer e gestos).
    pub fn live_replace_snapshot(&self) -> Result<Vec<u8>> {
        let id = self.shared()?;
        let p = self.catalog.photo(id).ok_or_else(|| err("sem foto"))?;
        serde_json::to_vec(&json!({ "develop": &*p.develop })).map_err(err)
    }

    pub fn live_digest(&self) -> Result<u64> {
        let id = self.shared()?;
        let p = self.catalog.photo(id).ok_or_else(|| err("sem foto"))?;
        Ok(fnv64(&serde_json::to_vec(&*p.develop).map_err(err)?))
    }

    pub fn live_load(&mut self, bytes: &[u8]) -> Result<()> {
        let v: Value = serde_json::from_slice(bytes).map_err(err)?;
        let develop: DevelopSettings = serde_json::from_value(v.get("develop").cloned().unwrap_or(Value::Null)).map_err(err)?;
        let id = match self.shared() {
            Ok(id) => id,
            Err(_) => self.live_receive_photo(&v)?,
        };
        self.set_develop(id, develop, "Colaboração")?;
        // Desfazer não pode apagar o ajuste do colega: o histórico local recomeça daqui.
        self.undo.clear();
        self.redo.clear();
        self.live.photo = Some(id);
        self.live.seen = self.develop_of(id);
        Ok(())
    }

    /// Convidado: traz a foto para a biblioteca (cena de demonstração ou arquivo recebido).
    fn live_receive_photo(&mut self, v: &Value) -> Result<PhotoId> {
        if let Some(scene) = v.get("demo").and_then(Value::as_u64) {
            let found = self.catalog.photos().find(|p| matches!(p.source, Source::Demo { scene: s } if u64::from(s) == scene)).map(|p| p.id);
            let id = found.ok_or_else(|| err("esta foto de demonstração não está na sua biblioteca"))?;
            return self.live_copy_photo(id);
        }
        let name = v.get("name").and_then(Value::as_str).unwrap_or("foto");
        let name: String = name.chars().filter(|c| !matches!(c, '/' | '\\' | ':' | '\0')).collect();
        let data = v.get("file").and_then(Value::as_str).ok_or_else(|| err("a foto não veio junto"))?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(data).map_err(err)?;
        let dir = std::env::temp_dir().join("nova-light-compartilhadas").join(format!("{:016x}", fnv64(&bytes)));
        std::fs::create_dir_all(&dir).map_err(err)?;
        let path = dir.join(if name.is_empty() { "foto".to_string() } else { name });
        std::fs::write(&path, &bytes).map_err(err)?;
        let report = crate::import::import(self, &[path.display().to_string()], crate::import::ImportMode::Copy)?;
        let id = report
            .imported
            .first()
            .copied()
            .or_else(|| report.duplicates.first().and_then(|d| d.existing))
            .map(PhotoId)
            .ok_or_else(|| err("não consegui importar a foto compartilhada"))?;
        self.live_copy_photo(id)
    }

    fn live_copy_photo(&mut self, id: PhotoId) -> Result<PhotoId> {
        let out = self.execute("photo.virtualCopy", &json!({"ids": [id.0], "name": "Colaboração"}))?;
        let id = out
            .get("ids")
            .and_then(Value::as_array)
            .and_then(|ids| ids.first())
            .and_then(Value::as_u64)
            .map(PhotoId)
            .ok_or_else(|| err("não foi possível criar a cópia compartilhada"))?;
        self.selection.active = Some(id);
        self.selection.ids = vec![id];
        Ok(id)
    }

    /// Roda o comando de um colega na foto compartilhada; seleção e desfazer locais ficam.
    pub fn live_apply(&mut self, cmd: &str, params: &Value, ctx: &Value) -> Result<()> {
        if !remote_command_allowed(cmd) {
            return Err(err("comando não permitido na colaboração"));
        }
        let id = self.shared()?;
        let params = match ctx.get("photo").and_then(Value::as_u64) {
            Some(theirs) => translate(params, theirs, id.0),
            None => params.clone(),
        };
        let local = self.selection.clone();
        self.selection.active = Some(id);
        self.selection.ids = vec![id];
        self.live.applying = true;
        let r = self.execute(cmd, &params);
        self.live.applying = false;
        // Desfazer não pode apagar o ajuste do colega: o histórico local recomeça daqui.
        self.undo.clear();
        self.redo.clear();
        self.selection = local;
        self.live.seen = self.develop_of(id);
        r.map(|_| ())
    }

    /// Chamado por `execute`: só o que mudou a revelação da foto compartilhada é replicado.
    pub(crate) fn live_record(&mut self, id: &str, params: &Value, journal: bool, ctx: Value) {
        if self.live.applying {
            return;
        }
        let Ok(photo) = self.shared() else { return };
        let now = self.develop_of(photo);
        if ptr_eq(&now, &self.live.seen) {
            return;
        }
        if journal && remote_command_allowed(id) && !matches!(id, "edit.undo" | "edit.redo") && self.interaction.is_none() {
            self.live.outbox.push(LiveChange::Op { cmd: id.to_string(), params: params.clone(), ctx });
            self.live.seen = now;
        }
    }

    pub(crate) fn live_context(&self) -> Option<Value> {
        self.live.photo.map(|p| json!({ "photo": p.0 }))
    }
}

fn ptr_eq(a: &Option<Arc<DevelopSettings>>, b: &Option<Arc<DevelopSettings>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn remote_commands_cannot_open_export_or_execute_code() {
        let (_, mut b) = pair();
        let before = b.live_digest().unwrap();
        for command in ["file.open", "file.saveAs", "file.export", "session.open", "script.run", "app.quit", "lisp.eval"] {
            assert!(b.live_apply(command, &serde_json::json!({"path": "/tmp/not-authorized"}), &serde_json::json!({})).is_err());
            assert_eq!(before, b.live_digest().unwrap());
        }
    }

    fn pair() -> (Session, Session) {
        let mut a = Session::with_demo();
        let first = a.catalog.photos().next().unwrap().id;
        a.selection.active = Some(first);
        a.selection.ids = vec![first];
        a.live_start().unwrap();
        let mut b = Session::with_demo();
        b.live_start_empty();
        let snap = a.live_snapshot().unwrap();
        b.live_load(&snap).unwrap();
        (a, b)
    }

    fn sync(a: &mut Session, b: &mut Session) {
        for change in a.live_take() {
            match change {
                LiveChange::Op { cmd, params, ctx } => b.live_apply(&cmd, &params, &ctx).unwrap(),
                LiveChange::Replace => {
                    let snap = a.live_replace_snapshot().unwrap();
                    b.live_load(&snap).unwrap();
                }
            }
        }
        assert!(b.live_take().is_empty(), "o que veio de A não volta para A");
    }

    #[test]
    fn develop_edits_replicate_on_the_shared_photo_only() {
        let (mut a, mut b) = pair();
        let b_undo = b.undo.len();
        a.execute("develop.adjust", &json!({"control": "wb.tint", "delta": 5})).unwrap();
        a.execute("develop.adjust", &json!({"control": "light.exposure", "delta": 0.5})).unwrap();
        sync(&mut a, &mut b);
        assert_eq!(a.live_digest().unwrap(), b.live_digest().unwrap());
        assert!(b.undo.is_empty(), "o desfazer de B recomeça depois do ajuste de A");
        let _ = b_undo;

        a.execute("edit.undo", &json!({})).unwrap();
        sync(&mut a, &mut b);
        assert_eq!(a.live_digest().unwrap(), b.live_digest().unwrap());

        b.execute("develop.adjust", &json!({"control": "wb.tint", "delta": -3})).unwrap();
        sync(&mut b, &mut a);
        assert_eq!(a.live_digest().unwrap(), b.live_digest().unwrap());
    }
}
