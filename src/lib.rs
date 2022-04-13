use std::num::NonZeroUsize;

/// `usize` 型の `0..n` の値を頂点とする functional graph。
/// レトロゲームの乱数調査などの用途を想定している。
///
/// functional graph の性質より、任意の頂点 v から辺を辿り続けると、必ずあるサイクルに達する。
/// これを「頂点 v が属するサイクル」と呼ぶ。
///
/// 個々のサイクルはその中の代表頂点で表す。サイクル中の最小の頂点を代表頂点とする。
#[derive(Debug)]
pub struct FunctionalGraph {
    /// 頂点数。
    n: usize,

    /// `succs[v]`: 頂点 `v` から 1 回辺を辿ったときの頂点。
    succs: Vec<usize>,

    /// `cycle_reprs[id]`: ID `id` のサイクルの代表頂点。
    cycle_reprs: Vec<usize>,

    /// `cycle_lens[id]`: ID `id` のサイクル内の頂点数。
    cycle_lens: Vec<NonZeroUsize>,

    /// `cycle_ids[v]`: 頂点 `v` が属するサイクルの ID。
    cycle_ids: Vec<usize>,

    /// `noncycle_lens[v]`: 頂点 `v` から辺を辿り続けたときの非循環節内の頂点数。
    noncycle_lens: Vec<usize>,

    /// source である頂点たち。
    sources: Vec<usize>,

    /// 各頂点が source かどうか。
    is_sources: Vec<bool>,
}

impl FunctionalGraph {
    /// 頂点数と遷移関数を指定して `FunctionalGraph` を作る。
    ///
    /// 値 `0..n` がグラフの頂点となる。
    ///
    /// 遷移関数 `f` は、`0..n` の範囲の引数に対し同じ範囲の一貫した値を返さねばならない。
    /// さもなくば結果は未定義。
    pub fn new(n: usize, f: impl FnMut(usize) -> usize) -> Self {
        let succs: Vec<_> = (0..n).map(f).collect();

        let (cycle_reprs, cycle_lens, cycle_ids, noncycle_lens) = Self::init_cycles(n, &succs);

        let (sources, is_sources) = Self::init_sources(n, &succs);

        Self {
            n,
            succs,
            cycle_reprs,
            cycle_lens,
            cycle_ids,
            noncycle_lens,
            sources,
            is_sources,
        }
    }

    fn init_cycles(
        n: usize,
        succs: &[usize],
    ) -> (Vec<usize>, Vec<NonZeroUsize>, Vec<usize>, Vec<usize>) {
        let mut cycle_reprs = Vec::<usize>::new();
        let mut cycle_lens = Vec::<NonZeroUsize>::new();
        let mut cycle_ids = vec![usize::MAX; n];
        let mut noncycle_lens = vec![usize::MAX; n];

        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum State {
            Unvisited,
            Visiting,
            Visited,
        }

        let mut cycle_id_nxt = 0;
        let mut states = vec![State::Unvisited; n];

        // Unvisited な頂点を順に取り出す (start とする)。
        // start からパスを辿ってサイクル情報を得る。
        for start in 0..n {
            if states[start] != State::Unvisited {
                debug_assert_ne!(states[start], State::Visiting);
                continue;
            }

            // パスを記録しつつ、辿った頂点の状態を Visiting に変更していく。
            // Unvisited でない頂点に達するまで続ける。このときの頂点を end とする。
            let mut path = Vec::<usize>::new();
            let end = {
                let mut v = start;
                while states[v] == State::Unvisited {
                    path.push(v);
                    states[v] = State::Visiting;
                    v = succs[v];
                }
                v
            };

            // end を始点としたときの (サイクルID, 非循環節長) を得る。
            // この過程で path のうちサイクル上にある頂点は消費される。
            let (cycle_id, mut ncl) = match states[end] {
                State::Visiting => {
                    // 今回のパスで新たなサイクルが作られており、end はそのサイクル上にある。
                    // サイクル上の頂点を逆順に取り出し、サイクル情報を更新していく。
                    let mut cycle_repr = usize::MAX;
                    let mut cycle_len = 0;
                    loop {
                        let v = path.pop().expect("v should be on this cycle");
                        states[v] = State::Visited;
                        cycle_ids[v] = cycle_id_nxt;
                        noncycle_lens[v] = 0;
                        cycle_repr = cycle_repr.min(v);
                        cycle_len += 1;
                        if v == end {
                            break;
                        }
                    }
                    // 新たなサイクルを追加。
                    cycle_reprs.push(cycle_repr);
                    cycle_lens
                        .push(NonZeroUsize::new(cycle_len).expect("cycle_len should not be 0"));
                    let cycle_id = cycle_id_nxt;
                    cycle_id_nxt += 1;
                    (cycle_id, 0)
                }
                State::Visited => {
                    // 今回のパスは既存のパスに合流している。
                    let cycle_id = cycle_ids[end];
                    (cycle_id, noncycle_lens[end])
                }
                State::Unvisited => unreachable!(), // ありえない(先ほどのループ終了条件より)。
            };

            // path に残った頂点についてサイクル情報を更新する。
            while let Some(v) = path.pop() {
                states[v] = State::Visited;
                ncl += 1;
                cycle_ids[v] = cycle_id;
                noncycle_lens[v] = ncl;
            }
        }

        (cycle_reprs, cycle_lens, cycle_ids, noncycle_lens)
    }

    fn init_sources(n: usize, succs: &[usize]) -> (Vec<usize>, Vec<bool>) {
        let mut is_sources = vec![true; n];

        for &succ in succs {
            is_sources[succ] = false;
        }

        let sources: Vec<_> = (0..n).filter(|&v| is_sources[v]).collect();

        (sources, is_sources)
    }

    /// 頂点数を返す。これは辺数に等しい。
    pub fn node_count(&self) -> usize {
        self.n
    }

    /// 指定した頂点から 1 回辺を辿ったときの頂点を返す。
    pub fn succ(&self, v: usize) -> usize {
        self.succs[v]
    }

    /// 指定した頂点から k 回辺を辿ったときの頂点を返す。
    pub fn kth_succ(&self, v: usize, k: usize) -> usize {
        // TODO: ダブリングで対数時間にできる

        // サイクル内を何度も周回するのは無駄なので、適切に剰余をとる。
        let k_opt = {
            let ncl = self.noncycle_len_of(v);
            if k >= ncl {
                let cl = self.cycle_len_of(v);
                ncl + (k - ncl) % cl
            } else {
                k
            }
        };

        (0..k_opt).fold(v, |v, _| self.succs[v])
    }

    /// 指定した頂点からサイクルを 1 周するまで辺を辿り続けたときの頂点列を生成する。
    ///
    /// 頂点列は先頭に `v` 自身を含む。また、サイクル上の各頂点はちょうど 1 回ずつ現れる。
    pub fn path_from(&self, v: usize) -> impl Iterator<Item = usize> + '_ {
        let count = self.noncycle_len_of(v) + self.cycle_len_of(v).get();

        std::iter::successors(Some(v), |&v| Some(self.succs[v])).take(count)
    }

    /// サイクルの個数を返す。これは弱連結成分の個数に等しい。
    pub fn cycle_count(&self) -> usize {
        self.cycle_reprs.len()
    }

    /// 全てのサイクルの代表頂点を列挙する。順序は未規定。
    pub fn cycles(&self) -> impl Iterator<Item = usize> + '_ {
        self.cycle_reprs.iter().copied()
    }

    /// 指定した頂点が属するサイクルの代表頂点を返す。
    pub fn cycle_of(&self, v: usize) -> usize {
        let id = self.cycle_ids[v];

        self.cycle_reprs[id]
    }

    /// 指定した頂点が属するサイクル内の頂点数を返す。
    pub fn cycle_len_of(&self, v: usize) -> NonZeroUsize {
        let id = self.cycle_ids[v];

        self.cycle_lens[id]
    }

    /// 指定した頂点から辺を辿り続けたときの非循環節内の頂点数を返す。
    pub fn noncycle_len_of(&self, v: usize) -> usize {
        self.noncycle_lens[v]
    }

    /// source の個数を返す。
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// 全ての source を昇順に列挙する。
    pub fn sources(&self) -> impl Iterator<Item = usize> + '_ {
        self.sources.iter().copied()
    }

    /// 指定した頂点が source かどうかを返す。
    pub fn is_source(&self, v: usize) -> bool {
        self.is_sources[v]
    }
}

#[cfg(test)]
mod tests {
    use itertools::{assert_equal, Itertools as _};

    use super::*;

    fn graph_from_succs(succs: impl AsRef<[usize]>) -> FunctionalGraph {
        _graph_from_succs(succs.as_ref())
    }

    fn _graph_from_succs(succs: &[usize]) -> FunctionalGraph {
        let n = succs.len();
        FunctionalGraph::new(n, |v| succs[v])
    }

    #[test]
    fn test_identity() {
        let fg = graph_from_succs([0, 1, 2]);

        assert_eq!(fg.node_count(), 3);

        assert_eq!(fg.succ(0), 0);
        assert_eq!(fg.succ(1), 1);
        assert_eq!(fg.succ(2), 2);

        assert_eq!(fg.kth_succ(0, 0), 0);
        assert_eq!(fg.kth_succ(0, 5), 0);

        assert_equal(fg.path_from(0), [0]);
        assert_equal(fg.path_from(1), [1]);
        assert_equal(fg.path_from(2), [2]);

        assert_eq!(fg.cycle_count(), 3);

        assert_equal(fg.cycles().sorted(), [0, 1, 2]);

        assert_eq!(fg.cycle_len_of(0), NonZeroUsize::new(1).unwrap());
        assert_eq!(fg.cycle_len_of(1), NonZeroUsize::new(1).unwrap());
        assert_eq!(fg.cycle_len_of(2), NonZeroUsize::new(1).unwrap());

        assert_eq!(fg.noncycle_len_of(0), 0);
        assert_eq!(fg.noncycle_len_of(1), 0);
        assert_eq!(fg.noncycle_len_of(2), 0);

        assert_eq!(fg.source_count(), 0);

        assert_equal(fg.sources(), []);

        assert!(!fg.is_source(0));
        assert!(!fg.is_source(1));
        assert!(!fg.is_source(2));
    }

    #[test]
    fn test_graph() {
        // 0 -> 1 ----> 2 <- 4
        //      ^       |
        //      |       |
        //      +-- 3 <-+
        let fg = graph_from_succs([1, 2, 3, 1, 2]);

        assert_eq!(fg.node_count(), 5);

        assert_eq!(fg.kth_succ(0, 11), 2);
        assert_eq!(fg.kth_succ(2, 5), 1);
        assert_eq!(fg.kth_succ(4, 3 * 1_000_000_000 + 2), 3);

        assert_equal(fg.path_from(0), [0, 1, 2, 3]);
        assert_equal(fg.path_from(2), [2, 3, 1]);
        assert_equal(fg.path_from(4), [4, 2, 3, 1]);

        assert_eq!(fg.cycle_count(), 1);

        assert_equal(fg.cycles(), [1]);

        assert_eq!(fg.cycle_of(0), 1);
        assert_eq!(fg.cycle_of(2), 1);
        assert_eq!(fg.cycle_of(4), 1);

        assert_eq!(fg.cycle_len_of(0), NonZeroUsize::new(3).unwrap());
        assert_eq!(fg.cycle_len_of(2), NonZeroUsize::new(3).unwrap());
        assert_eq!(fg.cycle_len_of(4), NonZeroUsize::new(3).unwrap());

        assert_eq!(fg.noncycle_len_of(0), 1);
        assert_eq!(fg.noncycle_len_of(2), 0);
        assert_eq!(fg.noncycle_len_of(4), 1);

        assert_eq!(fg.source_count(), 2);

        assert_equal(fg.sources(), [0, 4]);

        assert!(fg.is_source(0));
        assert!(!fg.is_source(2));
        assert!(fg.is_source(4));
    }
}
