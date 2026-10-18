use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TestCase {
    #[allow(unused)]
    pub name: String,
    pub initial: State,
    #[serde(rename = "final")] // final is a Rust keyword
    pub final_state: State,
    pub cycles: Vec<Cycle>,
}

#[derive(Debug, Deserialize)]
pub struct State {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    /// Each entry is a `[addr, value]` pair.
    pub ram: Vec<(u16, u8)>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CycleKind {
    Read,
    Write,
}

#[derive(Debug, Deserialize)]
#[serde(from = "(u16, u8, CycleKind)")]
pub struct Cycle {
    #[allow(unused)]
    pub addr: u16,
    #[allow(unused)]
    pub value: u8,
    #[allow(unused)]
    pub kind: CycleKind,
}

impl From<(u16, u8, CycleKind)> for Cycle {
    fn from((addr, value, kind): (u16, u8, CycleKind)) -> Self {
        Self { addr, value, kind }
    }
}
