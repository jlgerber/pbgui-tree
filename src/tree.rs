use crate::api::{ClientProxy, PackratDb};
use crate::inner_tree::InnerTreeView;
use qt_core::{QModelIndex, SlotOfQModelIndex};
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::{
    cpp_core::{MutPtr, Ref, StaticUpcast},
    QWidget,
};
use rustqt_utils::ToQStringOwned;
use std::cell::RefCell;
use std::rc::Rc;

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
            let treeview = Rc::new(RefCell::new(InnerTreeView::create(parent_widget)));
            let dtv = DistributionTreeView {
                view: treeview.clone(),
                // Slots
                clicked: SlotOfQModelIndex::new(move |_idx: Ref<QModelIndex>| {
                    //let parent = idx.parent();
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
        self.view.borrow_mut().set_default_stylesheet();
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
}
