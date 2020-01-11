use crate::api::{ClientProxy, PackratDb};
use crate::inner_tree::InnerTreeView;
use packybara::traits::*;
use qt_core::{QModelIndex, SlotOfQModelIndex};
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::{
    cpp_core::{CastInto, MutPtr, Ref, StaticUpcast},
    QComboBox, QFrame, QLabel, QLayout, QWidget,
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
    pub view: Rc<RefCell<InnerTreeView>>,
    pub clicked: SlotOfQModelIndex<'a>,
    pub expanded: SlotOfQModelIndex<'a>,
    pub collapsed: SlotOfQModelIndex<'a>,
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

            let cbox_p = Self::create_cbox(layout_ptr);
            let treeview = Rc::new(RefCell::new(InnerTreeView::create(qframe_ptr)));
            let tv = treeview.clone();
            let dtv = DistributionTreeView {
                parent_frame: qframe_ptr,
                view: treeview.clone(),
                cbox: cbox_p,
                // Slots
                clicked: SlotOfQModelIndex::new(move |_idx: Ref<QModelIndex>| {
                    tv.borrow_mut().clear_selection();
                }),

                expanded: SlotOfQModelIndex::new(
                    enclose! { (treeview) move |idx: Ref<QModelIndex>| {
                        println!("retrieving model");
                        let proxy_model = treeview.borrow().proxy_model();
                        let model = treeview.borrow().model();
                        println!("model retrieved");

                        let row_cnt = proxy_model.row_count_1a(idx);
                        if  row_cnt > 1 { return; }

                        // what if we only have 1 item? Lets make sure that it isnt
                        // an intended child (eg a single version or platform)
                        let child = idx.child(0,0);
                        println!("revrieving child item from index");

                        if !child.is_valid() || model.item_from_index(
                            proxy_model.map_to_source(child.as_ref()).as_mut_ref()
                        ).text().to_std_string() != "" {
                            return;
                        }
                        println!("retrieved child item from index");

                        println!("revrieving item from index");
                        let item = model.item_from_index(
                            proxy_model.map_to_source(idx).as_mut_ref()
                        );
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
            };

            // Set up signals & slots
            treeview.borrow().view.clicked().connect(&dtv.clicked);
            treeview.borrow().view.expanded().connect(&dtv.expanded);
            treeview.borrow().view.collapsed().connect(&dtv.collapsed);

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
    /// Set comboboc items, replacing any extant items
    ///
    /// # Arguments
    /// * `items` - Vector of items
    ///
    /// # Returns
    /// * None
    pub fn set_cb_items<'c, I>(&mut self, items: Vec<I>, current: I)
    where
        I: AsRef<str>,
    {
        unsafe {
            self.remove_cb_items();
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
    /// Remove all items from the combobox
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns None
    pub fn remove_cb_items(&mut self) {
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

    fn create_cbox<I>(layout: I) -> MutPtr<QComboBox>
    where
        I: CastInto<MutPtr<QLayout>>,
    {
        unsafe {
            // combo_box
            let mut horiz_frame = QFrame::new_0a();
            let mut h_layout = create_hlayout();
            let mut h_layout_p = h_layout.as_mut_ptr();
            horiz_frame.set_layout(h_layout.into_ptr());

            h_layout_p.add_stretch_1a(1);
            let site_l = QLabel::from_q_string(&qs("Site"));
            h_layout_p.add_widget(site_l.into_ptr());

            let mut cbox = QComboBox::new_0a();
            let cbox_p = cbox.as_mut_ptr();
            h_layout_p.add_widget(cbox.into_ptr());

            layout.cast_into().add_widget(horiz_frame.into_ptr());

            cbox_p
        }
    }
}
