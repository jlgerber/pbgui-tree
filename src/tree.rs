use qt_core::QAbstractItemModel;
use qt_core::{QModelIndex, SlotOfQModelIndex};
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::{
    cpp_core::{CastInto, CppBox, DynamicCast, MutPtr, Ref, StaticUpcast},
    QFrame, QMainWindow, QTreeView, QWidget,
};
use rustqt_utils::{qs, set_stylesheet_from_str, ToQStringOwned};

const STYLE_STR: &'static str = include_str!("../resources/tree.qss");

pub struct InnerTreeView {
    pub view: MutPtr<QTreeView>,
}

impl InnerTreeView {
    /// create a treeview
    pub fn create<T>(main_window: MutPtr<T>) -> InnerTreeView
    where
        T: StaticUpcast<QWidget>,
    {
        unsafe {
            let main_window = main_window.static_upcast_mut();
            let mut treeview = QTreeView::new_0a();
            let mut treeview_ptr = treeview.as_mut_ptr();
            treeview_ptr.set_root_is_decorated(true);
            treeview_ptr.set_items_expandable(true);
            treeview_ptr.set_uniform_row_heights(true);
            main_window.layout().add_widget(treeview.into_ptr());

            let mut model = QStandardItemModel::new_0a();
            model.set_column_count(1);
            treeview_ptr.set_model(model.into_ptr());

            InnerTreeView { view: treeview_ptr }
        }
    }

    /// Set the stylesheet to the internal stylesheet
    pub fn set_default_stylesheet(&mut self) {
        set_stylesheet_from_str(STYLE_STR, self.view);
    }
    /// Retreive the model from the view
    pub fn model(&mut self) -> MutPtr<QStandardItemModel> {
        unsafe {
            let model = self.view.model();
            if model.is_null() {
                panic!("Unable to retrieve modelfrom model pointer obtained via view.model()");
            }
            QAbstractItemModel::dynamic_cast_mut(model)
        }
    }

    /// Given a type that implements ToQstringOwned, append a distribution
    pub fn add_package<T: ToQStringOwned>(&mut self, input: T) {
        unsafe {
            let mut model = self.model();
            let row_count = model.row_count_0a();
            let mut parent = model.invisible_root_item();
            let mut item = QStandardItem::new();
            item.set_text(&input.to_qstring());
            item.set_editable(false);
            parent.append_row_q_standard_item(item.into_ptr());
            model.set_row_count(row_count + 1);
        }
    }

    pub fn clear_packages(&mut self) {
        unsafe {
            let mut model = self.model();
            for c in (0..model.row_count_0a()).rev() {
                model.clear_item_data(&self.model().index_2a(c, 0));
            }
            model.set_row_count(0)
        }
    }
    /// Given a vectro of a type that implements ToQstringOwned, append a distribution
    pub fn set_packages<T: ToQStringOwned>(&mut self, inputs: Vec<T>) {
        unsafe {
            let mut model = self.model();
            let mut parent = model.invisible_root_item();
            //model.clear(); // this removes columns as well. and segfaults
            let row_cnt = inputs.len() as i32;
            //
            for input in inputs {
                let mut item = QStandardItem::new();
                let txt = input.to_qstring();
                item.set_text(&txt);
                item.set_editable(false);
                // add one fake item to force qt to draw a
                let mut child = QStandardItem::new();
                child.set_text(&qs(""));
                child.set_editable(false);
                item.append_row_q_standard_item(child.into_ptr());
                parent.append_row_q_standard_item(item.into_ptr());
            }
            model.set_row_count(row_cnt);
        }
    }
    //pub fn set_d
    pub fn add_distribution<I>(&mut self, mut parent: MutPtr<qt_gui::QStandardItem>, version: I)
    where
        I: ToQStringOwned,
    {
        unsafe {
            let mut item = QStandardItem::new();
            let txt = version.to_qstring();
            item.set_text(&txt);
            item.set_editable(false);
            parent.append_row_q_standard_item(item.into_ptr());
        }
    }
}

pub struct DistributionTreeView<'a> {
    pub view: MutPtr<QTreeView>,
    pub clicked: SlotOfQModelIndex<'a>,
    pub expanded: SlotOfQModelIndex<'a>,
    pub collapsed: SlotOfQModelIndex<'a>,
}

impl<'a> DistributionTreeView<'a> {
    /// create a treeview
    pub fn create<T>(main_window: MutPtr<T>) -> DistributionTreeView<'a>
    where
        T: StaticUpcast<QWidget>,
    {
        unsafe {
            let main_window = main_window.static_upcast_mut();
            let mut treeview = QTreeView::new_0a();
            let mut treeview_ptr = treeview.as_mut_ptr();
            treeview_ptr.set_root_is_decorated(true);
            treeview_ptr.set_items_expandable(true);
            treeview_ptr.set_uniform_row_heights(true);
            main_window.layout().add_widget(treeview.into_ptr());

            let mut model = QStandardItemModel::new_0a();
            model.set_column_count(1);
            treeview_ptr.set_model(model.into_ptr());

            let dtv = DistributionTreeView {
                view: treeview_ptr,
                clicked: SlotOfQModelIndex::new(move |idx: Ref<QModelIndex>| {
                    println!("clicked {}", idx.row())
                }),
                expanded: SlotOfQModelIndex::new(move |idx: Ref<QModelIndex>| {
                    println!("expanded {}", idx.row());
                }),
                collapsed: SlotOfQModelIndex::new(move |idx: Ref<QModelIndex>| {
                    println!("collapsed {}", idx.row());
                }),
            };
            treeview_ptr.clicked().connect(&dtv.clicked);
            treeview_ptr.expanded().connect(&dtv.expanded);
            treeview_ptr.collapsed().connect(&dtv.collapsed);
            dtv
        }
    }

    /// Set the stylesheet to the internal stylesheet
    pub fn set_default_stylesheet(&mut self) {
        set_stylesheet_from_str(STYLE_STR, self.view);
    }
    /// Retreive the model from the view
    pub fn model(&mut self) -> MutPtr<QStandardItemModel> {
        unsafe {
            let model = self.view.model();
            if model.is_null() {
                panic!("Unable to retrieve modelfrom model pointer obtained via view.model()");
            }
            QAbstractItemModel::dynamic_cast_mut(model)
        }
    }

    /// Given a type that implements ToQstringOwned, append a distribution
    pub fn add_package<T: ToQStringOwned>(&mut self, input: T) {
        unsafe {
            let mut model = self.model();
            let row_count = model.row_count_0a();
            let mut parent = model.invisible_root_item();
            let mut item = QStandardItem::new();
            item.set_text(&input.to_qstring());
            item.set_editable(false);
            parent.append_row_q_standard_item(item.into_ptr());
            model.set_row_count(row_count + 1);
        }
    }

    pub fn clear_packages(&mut self) {
        unsafe {
            let mut model = self.model();
            for c in (0..model.row_count_0a()).rev() {
                model.clear_item_data(&self.model().index_2a(c, 0));
            }
            model.set_row_count(0)
        }
    }
    /// Given a vectro of a type that implements ToQstringOwned, append a distribution
    pub fn set_packages<T: ToQStringOwned>(&mut self, inputs: Vec<T>) {
        unsafe {
            let mut model = self.model();
            let mut parent = model.invisible_root_item();
            //model.clear(); // this removes columns as well. and segfaults
            let row_cnt = inputs.len() as i32;
            //
            for input in inputs {
                let mut item = QStandardItem::new();
                let txt = input.to_qstring();
                item.set_text(&txt);
                item.set_editable(false);
                // add one fake item to force qt to draw a
                let mut child = QStandardItem::new();
                child.set_text(&qs(""));
                child.set_editable(false);
                item.append_row_q_standard_item(child.into_ptr());
                parent.append_row_q_standard_item(item.into_ptr());
            }
            model.set_row_count(row_cnt);
        }
    }
    //pub fn set_d
    pub fn add_distribution<I>(&mut self, mut parent: MutPtr<qt_gui::QStandardItem>, version: I)
    where
        I: ToQStringOwned,
    {
        unsafe {
            let mut item = QStandardItem::new();
            let txt = version.to_qstring();
            item.set_text(&txt);
            item.set_editable(false);
            parent.append_row_q_standard_item(item.into_ptr());
        }
    }
}
