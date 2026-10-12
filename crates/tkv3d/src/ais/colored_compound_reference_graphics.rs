//! Named driver delegate observes actual WF callback ownership and injects only a checked graphics allocation failure.
use super::{
    AisColoredDrawerCompounds, colored_compound_reference_group::ColoredCompoundReferenceGroupFake,
};
use rustycad_tkbrep::topods::Compound;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use std::{
    fmt::Write,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
pub(super) struct ColoredCompoundGraphicsTrace {
    pub rows: Mutex<String>,
    pub fail: AtomicBool,
    attempts: AtomicUsize,
    replacement: Option<Weak<ColoredCompoundLiveFixture>>,
    map: Weak<AisColoredDrawerCompounds>,
}
impl ColoredCompoundGraphicsTrace {
    pub(super) fn new(map: &Arc<AisColoredDrawerCompounds>, fail: bool) -> Arc<Self> {
        Arc::new(Self {
            rows: Default::default(),
            fail: AtomicBool::new(fail),
            attempts: AtomicUsize::new(0),
            replacement: None,
            map: Arc::downgrade(map),
        })
    }
    pub(super) fn with_replacement(
        map: &Arc<AisColoredDrawerCompounds>,
        fixture: &Arc<ColoredCompoundLiveFixture>,
    ) -> Arc<Self> {
        Arc::new(Self {
            rows: Default::default(),
            fail: AtomicBool::new(false),
            attempts: AtomicUsize::new(0),
            map: Arc::downgrade(map),
            replacement: Some(Arc::downgrade(fixture)),
        })
    }
    fn record_group(&self) -> Result<(), Failure> {
        let map = self
            .map
            .upgrade()
            .expect("map expired; expected live native WF Value");
        let count = map.node_ref(1)?.compound_ref_count()?;
        let attempt = self.attempts.fetch_add(1, Ordering::Relaxed) + 1;
        writeln!(self.rows.lock().unwrap(), "wf-group {attempt} {count}").unwrap();
        if self.fail.load(Ordering::Relaxed) {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "group index=1; expected available native group",
                "",
            ));
        }
        if attempt == 1 {
            self.apply_replacement(&map)?;
        }
        Ok(())
    }
    fn apply_replacement(&self, map: &AisColoredDrawerCompounds) -> Result<(), Failure> {
        let Some(weak) = &self.replacement else {
            return Ok(());
        };
        let fixture = weak
            .upgrade()
            .expect("fixture expired; expected live callback input");
        let alias = map.node_ref(1)?;
        alias.assign_compound(&fixture.incoming)?;
        let same = alias.with_shape(|shape| Ok(shape.is_same(&fixture.incoming)))?;
        let offset = fixture
            .incoming
            .location()
            .transformation()
            .translation_part();
        writeln!(
            self.rows.lock().unwrap(),
            "wf-value-assigned {} {} {} {} {} {}",
            fixture.original.tshape().ref_count(),
            fixture.incoming.tshape().ref_count(),
            i32::from(same),
            offset.x(),
            offset.y(),
            offset.z()
        )
        .unwrap();
        Ok(())
    }
}
pub(super) struct ColoredCompoundLiveFixture {
    pub original: Compound,
    pub incoming: Compound,
}
pub(super) fn compound_reference_presentation(
    trace: &Arc<ColoredCompoundGraphicsTrace>,
) -> Handle<dyn Graphic3dStructureSource> {
    let driver = NullGraphicDriver::new_handle();
    let owner = Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
    let prs = Graphic3dStructure::new(&owner, &Handle::default()).unwrap();
    let wrapper = ColoredCompoundReferenceStructureFake {
        transient: Default::default(),
        inner: prs.cstructure(),
        trace: Arc::clone(trace),
        _owner: owner,
    };
    prs.structure_base().fields.write().cstructure = Handle::new(wrapper).cast();
    prs.cast()
}
struct ColoredCompoundReferenceStructureFake {
    transient: StandardTransient,
    inner: Handle<dyn Graphic3dCStructure>,
    trace: Arc<ColoredCompoundGraphicsTrace>,
    _owner: Handle<Graphic3dStructureManager>,
}
rustycad_tkernel::impl_transient!(ColoredCompoundReferenceStructureFake,"Martensite_ColoredCompoundReferenceStructure",StandardTransient,transient,interfaces:[dyn Graphic3dCStructure]);
impl Graphic3dCStructure for ColoredCompoundReferenceStructureFake {
    fn cstructure_base(&self) -> &Graphic3dCStructureBase {
        let structure = &self.inner;
        structure.cstructure_base()
    }
    fn on_visibility_changed(&self) {
        let structure = &self.inner;
        structure.on_visibility_changed();
    }
    fn clear(&self) {
        let structure = &self.inner;
        structure.clear();
    }
    fn connect(&self, other: &dyn Graphic3dCStructure) {
        let structure = &self.inner;
        structure.connect(other);
    }
    fn disconnect(&self, other: &dyn Graphic3dCStructure) {
        let structure = &self.inner;
        structure.disconnect(other);
    }
    fn graphic_highlight(&self, style: &Handle<dyn Graphic3dPresentationAttributesSource>) {
        let structure = &self.inner;
        structure.graphic_highlight(style);
    }
    fn graphic_unhighlight(&self) {
        let structure = &self.inner;
        structure.graphic_unhighlight();
    }
    fn shadow_link(
        &self,
        owner: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        let structure = &self.inner;
        structure.shadow_link(owner)
    }
    fn remove_group(&self, group: &Handle<dyn Graphic3dGroup>) {
        let structure = &self.inner;
        structure.remove_group(group);
    }
    fn new_group(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        self.trace.record_group()?;
        if self.trace.replacement.is_none() {
            return self.inner.new_group(prs);
        }
        let group = Handle::new(ColoredCompoundReferenceGroupFake::new(prs, &self.trace)).cast();
        self.cstructure_base()
            .fields
            .write()
            .groups
            .push(Handle::clone(&group));
        Ok(group)
    }
}
