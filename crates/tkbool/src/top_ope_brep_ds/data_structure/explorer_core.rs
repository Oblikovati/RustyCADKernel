// Shared translation of the PointExplorer and CurveExplorer traversal at
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{TopOpeBRepDSCurve, TopOpeBRepDSDataStructure, TopOpeBRepDSPoint};

pub(super) trait ExplorerGeometry: Default {
    fn count(source: &TopOpeBRepDSDataStructure) -> i32;
    fn lookup(source: &TopOpeBRepDSDataStructure, index: i32) -> Option<&Self>;
    fn kept(&self) -> bool;
}

impl ExplorerGeometry for TopOpeBRepDSCurve {
    fn count(source: &TopOpeBRepDSDataStructure) -> i32 {
        source.nb_curves()
    }
    fn lookup(source: &TopOpeBRepDSDataStructure, index: i32) -> Option<&Self> {
        source.my_curves.seek(&index).map(|entry| &entry.my_curve)
    }
    fn kept(&self) -> bool {
        self.keep()
    }
}

impl ExplorerGeometry for TopOpeBRepDSPoint {
    fn count(source: &TopOpeBRepDSDataStructure) -> i32 {
        source.nb_points()
    }
    fn lookup(source: &TopOpeBRepDSDataStructure, index: i32) -> Option<&Self> {
        source.my_points.seek(&index).map(|entry| &entry.my_point)
    }
    fn kept(&self) -> bool {
        self.keep()
    }
}

enum GeometryExplorerSource<'a> {
    Shared(&'a TopOpeBRepDSDataStructure),
    Exclusive(&'a mut TopOpeBRepDSDataStructure),
}

impl GeometryExplorerSource<'_> {
    fn as_ref(&self) -> &TopOpeBRepDSDataStructure {
        match self {
            Self::Shared(source) => source,
            Self::Exclusive(source) => source,
        }
    }
}

pub(super) struct GeometryExplorer<'a, G: ExplorerGeometry> {
    source: Option<GeometryExplorerSource<'a>>,
    index: i32,
    maximum: i32,
    found: bool,
    find_keep: bool,
    empty: G,
}

impl<G: ExplorerGeometry> Default for GeometryExplorer<'_, G> {
    fn default() -> Self {
        Self {
            source: None,
            index: 1,
            maximum: 0,
            found: false,
            find_keep: false,
            empty: G::default(),
        }
    }
}

impl<'a, G: ExplorerGeometry> GeometryExplorer<'a, G> {
    pub(super) fn new(source: &'a TopOpeBRepDSDataStructure, find_keep: bool) -> Self {
        let mut explorer = Self::default();
        explorer.init(source, find_keep);
        explorer
    }

    pub(super) fn init(&mut self, source: &'a TopOpeBRepDSDataStructure, find_keep: bool) {
        self.source = Some(GeometryExplorerSource::Shared(source));
        self.restart(find_keep);
    }

    pub(super) fn new_mut(source: &'a mut TopOpeBRepDSDataStructure, find_keep: bool) -> Self {
        let mut explorer = Self {
            source: Some(GeometryExplorerSource::Exclusive(source)),
            ..Self::default()
        };
        explorer.restart(find_keep);
        explorer
    }

    pub(super) fn source_mut(&mut self) -> Option<&mut TopOpeBRepDSDataStructure> {
        match self.source.as_mut()? {
            GeometryExplorerSource::Exclusive(source) => Some(source),
            GeometryExplorerSource::Shared(_) => None,
        }
    }

    fn restart(&mut self, find_keep: bool) {
        self.index = 1;
        self.maximum = self.source().map_or(0, G::count);
        self.find_keep = find_keep;
        self.find();
    }

    pub(super) fn more(&self) -> bool {
        self.found
    }

    pub(super) fn next(&mut self) {
        self.index += 1;
        self.find();
    }

    pub(super) fn current_index(&self) -> Option<i32> {
        self.found.then_some(self.index)
    }

    pub(super) fn empty(&self) -> &G {
        &self.empty
    }

    pub(super) fn is_bound(&self, index: i32) -> bool {
        self.source()
            .and_then(|source| G::lookup(source, index))
            .is_some()
    }

    pub(super) fn is_kept(&self, index: i32) -> bool {
        self.source()
            .and_then(|source| G::lookup(source, index))
            .is_some_and(G::kept)
    }

    pub(super) fn at(&self, index: i32) -> &G {
        // The native explorer's fallback is separate from DataStructure::ChangeCurve's
        // mutable fallback, which callers can overwrite through an unknown index.
        // PointExplorer follows the same rule, with its own myEmpty point.
        self.source()
            .and_then(|source| G::lookup(source, index))
            .unwrap_or(&self.empty)
    }

    pub(super) fn count(&mut self) -> i32 {
        self.restart(self.find_keep);
        let mut count: i32 = 0;
        while self.more() {
            count += 1;
            self.next();
        }
        count
    }

    pub(super) fn index(&self) -> i32 {
        self.index
    }

    pub(super) fn source(&self) -> Option<&TopOpeBRepDSDataStructure> {
        self.source.as_ref().map(GeometryExplorerSource::as_ref)
    }

    fn find(&mut self) {
        self.found = false;
        while self.index <= self.maximum {
            self.found = self.is_bound(self.index) && (!self.find_keep || self.is_kept(self.index));
            if self.found {
                return;
            }
            self.index += 1;
        }
    }
}
