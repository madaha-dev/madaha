/// Multi EQ 配置结构体
/// 每个 EQ 包含 5 个频段，每个频段有：Gain、Frequency、Q、Shape
#[derive(Debug, Clone, Copy)]
pub struct MultiEQ {
    pub band1: EQBand,
    pub band2: EQBand,
    pub band3: EQBand,
    pub band4: EQBand,
    pub band5: EQBand,
}

/// EQ 单频段配置
#[derive(Debug, Clone, Copy)]
pub struct EQBand {
    pub gain: u16,
    pub frequency: u16,
    pub q: u16,
    pub shape: u16,
}

// Flat EQ 预设
pub const MULTI_EQ_FLAT: MultiEQ = MultiEQ {
    band1: EQBand {
        gain: 64,
        frequency: 12,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 64,
        frequency: 28,
        q: 7,
        shape: 0,
    },
    band3: EQBand {
        gain: 64,
        frequency: 34,
        q: 7,
        shape: 0,
    },
    band4: EQBand {
        gain: 64,
        frequency: 46,
        q: 7,
        shape: 0,
    },
    band5: EQBand {
        gain: 64,
        frequency: 52,
        q: 7,
        shape: 0,
    },
};

// Jazz EQ 预设
pub const MULTI_EQ_JAZZ: MultiEQ = MultiEQ {
    band1: EQBand {
        gain: 58,
        frequency: 8,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 66,
        frequency: 16,
        q: 3,
        shape: 0,
    },
    band3: EQBand {
        gain: 68,
        frequency: 33,
        q: 3,
        shape: 0,
    },
    band4: EQBand {
        gain: 60,
        frequency: 44,
        q: 5,
        shape: 0,
    },
    band5: EQBand {
        gain: 58,
        frequency: 50,
        q: 7,
        shape: 0,
    },
};

// Pops EQ 预设
pub const MULTI_EQ_POPS: MultiEQ = MultiEQ {
    band1: EQBand {
        gain: 68,
        frequency: 16,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 60,
        frequency: 24,
        q: 20,
        shape: 0,
    },
    band3: EQBand {
        gain: 67,
        frequency: 34,
        q: 7,
        shape: 0,
    },
    band4: EQBand {
        gain: 60,
        frequency: 40,
        q: 20,
        shape: 0,
    },
    band5: EQBand {
        gain: 70,
        frequency: 48,
        q: 7,
        shape: 0,
    },
};

// Rock EQ 预设
pub const MULTI_EQ_ROCK: MultiEQ = MultiEQ {
    band1: EQBand {
        gain: 71,
        frequency: 16,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 68,
        frequency: 20,
        q: 7,
        shape: 0,
    },
    band3: EQBand {
        gain: 60,
        frequency: 36,
        q: 5,
        shape: 0,
    },
    band4: EQBand {
        gain: 68,
        frequency: 41,
        q: 10,
        shape: 0,
    },
    band5: EQBand {
        gain: 66,
        frequency: 50,
        q: 7,
        shape: 0,
    },
};

// Concert EQ 预设
pub const MULTI_EQ_CONCERT: MultiEQ = MultiEQ {
    band1: EQBand {
        gain: 67,
        frequency: 12,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 68,
        frequency: 24,
        q: 7,
        shape: 0,
    },
    band3: EQBand {
        gain: 64,
        frequency: 34,
        q: 5,
        shape: 0,
    },
    band4: EQBand {
        gain: 66,
        frequency: 50,
        q: 7,
        shape: 0,
    },
    band5: EQBand {
        gain: 61,
        frequency: 52,
        q: 7,
        shape: 0,
    },
};
