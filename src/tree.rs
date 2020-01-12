use crate::api::{ClientProxy, PackratDb};
use crate::inner_tree::InnerTreeView;
use packybara::traits::*;
use qt_core::{QModelIndex, QSize, SlotOfBool, SlotOfQModelIndex};
use qt_gui::{
    q_icon::{Mode, State},
    QIcon, QStandardItem, QStandardItemModel,
};
use qt_widgets::{
    cpp_core::{CastInto, MutPtr, Ref, StaticUpcast},
    QComboBox, QFrame, QLabel, QLayout, QPushButton, QWidget,
};

use rustqt_utils::{create_hlayout, create_vlayout, qs, set_stylesheet_from_str, ToQStringOwned};
use std::cell::RefCell;
use std::rc::Rc;

const STYLE_STR: &'static str = include_str!("../resources/tree.qss");

//makes it simpler to deal with the need to clone. Saw this here:
// https://github.com/rust-webplatform/rust-todomvc/blob/master/src/main.rs#L142
macro_rules! enclose {
    ( ($(  $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub struct DistributionTreeView<'a> {
    pub parent_frame: MutPtr<QFrame>,
    pub cbox: MutPtr<QComboBox>,
    pub filter_cb: MutPtr<QPushButton>,
    pub view: Rc<RefCell<InnerTreeView<'a>>>,
    pub clicked: SlotOfQModelIndex<'a>,
    pub expanded: SlotOfQModelIndex<'a>,
    pub collapsed: SlotOfQModelIndex<'a>,
    pub filter_visible: SlotOfBool<'a>,
}

// filter using is any
fn is_not_any(item: &str) -> Option<&str> {
    if item == "any" {
        None
    } else {
        Some(item)
    }
}

impl<'a> DistributionTreeView<'a> {
    /// create a treeview given a main window of any type that can be cast to QWidget
    ///
    /// # Arguments
    /// * `parent_widget` - The parent of the tree view
    ///
    /// # Returns
    /// * `DistributionTreeView instance
    pub fn create<T>(parent_widget: MutPtr<T>) -> DistributionTreeView<'a>
    where
        T: StaticUpcast<QWidget>,
    {
        unsafe {
            let mut qframe = QFrame::new_0a();
            let qframe_ptr = qframe.as_mut_ptr();
            let mut layout = create_vlayout();
            let layout_ptr = layout.as_mut_ptr();
            qframe.set_layout(layout.into_ptr());

            let parent_widget = parent_widget.static_upcast_mut();
            parent_widget.layout().add_widget(qframe.into_ptr());

            let (cbox_p, filter_btn) = Self::create_cbox(layout_ptr);
            let treeview = Rc::new(RefCell::new(InnerTreeView::create(qframe_ptr)));
            let tv = treeview.clone();

            let dtv = DistributionTreeView {
                parent_frame: qframe_ptr,
                view: treeview.clone(),
                cbox: cbox_p,
                filter_cb: filter_btn,
                // Slots
                clicked: SlotOfQModelIndex::new(move |_idx: Ref<QModelIndex>| {
                    tv.borrow_mut().clear_selection();
                }),

                expanded: SlotOfQModelIndex::new(
                    enclose! { (treeview) move |idx: Ref<QModelIndex>| {
                        let model = treeview.borrow().model();
                        let row_cnt = model.row_count_1a(idx);
                        if  row_cnt > 1 { return; }

                        // what if we only have 1 item? Lets make sure that it isnt
                        // an intended child (eg a single version or platform)
                        let child = idx.child(0,0);
                        if !child.is_valid() || model.item_from_index(child.as_ref()).text().to_std_string() != "" {
                            return;
                        }

                        let item = model.item_from_index(idx);
                        let item_str = item.text().to_std_string();

                        let client = ClientProxy::connect().expect("Unable to connect via ClientProxy");
                        let mut db = PackratDb::new(client);

                        // we are a child of the root. Our parent is not "valid"
                        if idx.parent().is_valid() == false {
                            let results = db
                                .find_all_distributions()
                                .package(&item_str)
                                .query()
                                .expect("unable to find_all_distributions");
                            let results = results.iter().map(|s| s.version.as_str()).collect::<Vec<_>>();
                            if results.len() > 0 {
                                treeview.borrow_mut().model().remove_rows_3a(0,1, idx);
                                treeview.borrow_mut().set_children(item, results, true);
                            }
                        } else {
                            // if we are not the child of the root, we must be the version, revealing
                            // the platform
                            let results = db
                                .find_all_platforms()
                                .query()
                                .expect("unable to find_all_platforms");
                            let results = results.iter().filter_map(|s| is_not_any(s.name.as_str())).collect::<Vec<_>>();
                            if results.len() > 0 {
                                treeview.borrow_mut().model().remove_rows_3a(0,1, idx);
                                treeview.borrow_mut().set_children(item, results, false);
                            }
                        }
                    }},
                ),

                collapsed: SlotOfQModelIndex::new(
                    enclose! { (treeview) move |idx: Ref<QModelIndex>| {
                        if treeview.borrow().model().row_count_1a(idx) == 1 {
                            treeview.borrow_mut().view.set_row_hidden(0, idx, false);
                        }
                    }},
                ),
                filter_visible: SlotOfBool::new(enclose! { (treeview) move |vis: bool| {
                    treeview.borrow_mut().set_filter_visibility(vis);
                }}),
            };

            // Set up signals & slots
            treeview.borrow().view.clicked().connect(&dtv.clicked);
            treeview.borrow().view.expanded().connect(&dtv.expanded);
            treeview.borrow().view.collapsed().connect(&dtv.collapsed);
            dtv.filter_cb.toggled().connect(&dtv.filter_visible);
            dtv
        }
    }

    /// Set the stylesheet to the internal stylesheet
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// *None
    pub fn set_default_stylesheet(&mut self) {
        set_stylesheet_from_str(STYLE_STR, self.parent_frame);
    }

    /// Retreive the model from the view
    ///
    /// # Aeguments
    /// * None
    ///
    /// # Returns
    /// * A mutable pointer to the QStandardItemModel
    pub fn model(&self) -> MutPtr<QStandardItemModel> {
        self.view.borrow().model()
    }

    /// Given a type that implements ToQstringOwned, append a distribution.
    ///
    /// # Arguments
    /// * `input` - Instance of any type that implements the ToQStringOwned trait.
    /// (this includes &str, String and QString)
    ///
    /// # Returns
    /// * None
    pub fn add_package<T: ToQStringOwned>(&mut self, input: T) {
        self.view.borrow_mut().add_package(input);
    }

    /// Clear the list of packages
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// * None
    pub fn clear_packages(&mut self) {
        self.view.borrow_mut().clear_packages();
    }

    pub fn clear_selection(&self) {
        unsafe {
            self.view
                .borrow_mut()
                .view
                .selection_model()
                .clear_selection();
        }
    }

    /// Given a vector of a type that implements the ToQstringOwned trait, set the packages
    /// to match the list.
    ///
    /// # Arguments
    /// * `inputs` - A vecctor of package names (&str or String or QString or...)
    ///
    /// # Returns
    /// * None
    pub fn set_packages<T: ToQStringOwned>(&mut self, inputs: Vec<T>) {
        self.view.borrow_mut().set_packages(inputs);
    }

    /// Add a child to the provided parent.
    ///
    /// # Arguments
    /// * `parent` - a mutable pointer to a QStandardItem rep of a package
    /// * `child` - a disribution version, represented by any type implementing the ToQStringOwned trait.
    ///
    /// # Returns
    /// * None
    pub fn add_child<I>(&mut self, parent: MutPtr<QStandardItem>, child: I)
    where
        I: ToQStringOwned,
    {
        self.view.borrow_mut().add_child(parent, child);
    }

    #[allow(dead_code)]
    /// Set combobox sites, replacing any extant sites
    ///
    /// # Arguments
    /// * `items` - Vector of items
    ///
    /// # Returns
    /// * None
    pub fn set_sites<'c, I>(&mut self, items: Vec<I>, current: I)
    where
        I: AsRef<str>,
    {
        unsafe {
            self.remove_sites();
            let mut idx = 0;
            let mut cnt = 0;
            for item in items {
                if current.as_ref() == item.as_ref() {
                    idx = cnt;
                }
                self.cbox.add_item_q_string(&qs(item.as_ref()));
                cnt += 1;
            }
            self.cbox.set_current_index(idx);
        }
    }

    #[allow(dead_code)]
    /// Remove all sites from the combobox
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns None
    pub fn remove_sites(&mut self) {
        unsafe {
            self.cbox.clear();
        }
    }

    /// Change the max number of items displayed in the combobox's dropdown
    /// list
    ///
    /// # Arguments
    /// * `max` - Maximum number of visible items in the comobobox's dropdown
    ///
    /// # Returns
    /// * None
    pub fn set_cb_max_visible_items(&mut self, max: i32) {
        unsafe {
            self.cbox.set_max_visible_items(max);
        }
    }

    fn create_cbox<I>(layout: I) -> (MutPtr<QComboBox>, MutPtr<QPushButton>)
    where
        I: CastInto<MutPtr<QLayout>>,
    {
        unsafe {
            // combo_box
            let mut horiz_frame = QFrame::new_0a();
            horiz_frame.set_object_name(&qs("SitesCBFrame"));
            let mut h_layout = create_hlayout();
            let mut h_layout_p = h_layout.as_mut_ptr();
            horiz_frame.set_layout(h_layout.into_ptr());

            let mut site_l = QLabel::from_q_string(&qs("Site"));
            site_l.set_object_name(&qs("SiteLabel"));
            let mut icon = QIcon::new();
            icon.add_file_2a(&qs(":/images/world.svg"), QSize::new_2a(12, 12).as_ref());
            let pixmap = icon.pixmap_int(12);
            site_l.set_pixmap(&pixmap);
            h_layout_p.add_stretch_1a(1);

            h_layout_p.add_widget(site_l.into_ptr());

            let mut cbox = QComboBox::new_0a();
            let cbox_p = cbox.as_mut_ptr();
            h_layout_p.add_widget(cbox.into_ptr());

            let mut filter_btn = QPushButton::new();
            let filter_btn_ptr = filter_btn.as_mut_ptr();
            filter_btn.set_object_name(&qs("packageFilterCheckbox"));
            filter_btn.set_checkable(true);
            filter_btn.set_tool_tip(&qs("Display the Package filter control"));
            let mut icon = QIcon::new();
            icon.add_file_2a(
                &qs(":/images/filter_white_sm.svg"),
                QSize::new_2a(10, 10).as_ref(),
            );
            icon.add_file_4a(
                &qs(":/images/filter_blue_sm.svg"),
                QSize::new_2a(10, 10).as_ref(),
                Mode::Normal,
                State::On,
            );
            filter_btn.set_icon(&icon);
            h_layout_p.add_widget(filter_btn.into_ptr());
            layout.cast_into().add_widget(horiz_frame.into_ptr());

            (cbox_p, filter_btn_ptr)
        }
    }
}
