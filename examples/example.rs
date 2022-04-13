use functional_graph::*;

fn main() {
    summarize("DQ1 RNG", &graph_u16(dq1_rng));
    summarize("DQ2 RNG", &graph_u16(dq2_rng));
    summarize("Wiz3 RNG (use)", &graph_u16(wiz3_rng_use));
    summarize("Wiz3 RNG (wait)", &graph_u16(wiz3_rng_wait));
}

fn summarize(name: &str, fg: &FunctionalGraph) {
    let cycles: Vec<_> = fg.cycles().collect();
    let sources: Vec<_> = fg.sources().collect();

    println!("[{name}]");
    println!("node count: {}", fg.node_count());
    println!("cycle count: {}", fg.cycle_count());
    for (i, &repr) in cycles.iter().enumerate() {
        println!("cycle {i}:");
        println!("  repr: {repr}");
        println!("  len: {}", fg.cycle_len_of(repr));
    }
    println!("source count: {}", fg.source_count());
    println!("sources: {:?}", sources);

    println!();
}

/// `u16` 上の `FunctionalGraph` を作る。
fn graph_u16(mut f: impl FnMut(u16) -> u16) -> FunctionalGraph {
    FunctionalGraph::new(0x10000, |v| {
        let v = u16::try_from(v).expect("v should be u16");
        let succ = f(v);
        usize::from(succ)
    })
}

/// ドラゴンクエスト (FC) の乱数生成器。
fn dq1_rng(r: u16) -> u16 {
    r.wrapping_mul(771).wrapping_add(129)
}

/// ドラゴンクエスト2 (FC) の乱数生成器。
fn dq2_rng(r: u16) -> u16 {
    fn crc_update(crc: u16) -> u16 {
        let mut reg = crc ^ 0xFF00;

        for _ in 0..8 {
            let carry = (reg & 0x8000) != 0;
            reg <<= 1;
            if carry {
                reg ^= 0x1021;
            }
        }

        reg
    }

    crc_update(crc_update(r))
}

/// ウィザードリィ3 (FC) の乱数生成器 A (乱数使用時の更新式)。
fn wiz3_rng_use(r: u16) -> u16 {
    r.wrapping_mul(257).wrapping_add(1)
}

/// ウィザードリィ3 (FC) の乱数生成器 B (割り込み待ち中の更新式)。
fn wiz3_rng_wait(r: u16) -> u16 {
    let mut lo = u8::try_from(r & 0xFF).unwrap();
    let mut hi = u8::try_from(r >> 8).unwrap();

    lo = lo.wrapping_add(1);
    hi = hi.wrapping_mul(5).wrapping_add(1);

    u16::from(lo) | (u16::from(hi) << 8)
}
