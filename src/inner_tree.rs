use qt_core::QAbstractItemModel;
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::{
    cpp_core::{DynamicCast, MutPtr, StaticUpcast},
    QTreeView, QWidget,
};
use rustqt_utils::{qs, set_stylesheet_from_str, ToQStringOwned};

const STYLE_STR: &'static str = include_str!("../resources/tree.qss");

/// A struct holding the QTreeView and providing a simple Api, mirrored
/// by the parent.
pub struct InnerTreeView {
    pub view: MutPtr<QTreeView>,
}

impl InnerTreeView {
    /// create an InnerTreeView instance. This inner tree allows us
    /// to use the tree's api in Slots exposed by the parent
    pub fn create<T>(parent_widget: MutPtr<T>) -> InnerTreeView
    where
        T: StaticUpcast<QWidget>,
    {
        unsafe {
            let parent_widget = parent_widget.static_upcast_mut();
            let mut treeview = QTreeView::new_0a();
            let mut treeview_ptr = treeview.as_mut_ptr();
            treeview_ptr.set_root_is_decorated(true);
            treeview_ptr.set_items_expandable(true);
            treeview_ptr.set_uniform_row_heights(true);
            parent_widget.layout().add_widget(treeview.into_ptr());

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
    pub fn model(&self) -> MutPtr<QStandardItemModel> {
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

    /// Given a vector of a type that implements ToQstringOwned, append a distribution
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

    /// Add a child to the tree.
    ///
    /// # Arguments
    /// * `parent` The parent QStandardItem
    /// * `child` - A value of any type implementing the ToQStringOwned trait.
    ///
    /// # Returns
    /// * None
    pub fn add_child<I>(&mut self, mut parent: MutPtr<qt_gui::QStandardItem>, child: I)
    where
        I: ToQStringOwned,
    {
        unsafe {
            let mut item = QStandardItem::new();
            let txt = child.to_qstring();
            item.set_text(&txt);
            item.set_editable(false);
            parent.append_row_q_standard_item(item.into_ptr());
        }
    }

    pub fn set_children<I>(
        &mut self,
        mut parent: MutPtr<qt_gui::QStandardItem>,
        children: Vec<I>,
        add_empty_gchild: bool,
    ) where
        I: ToQStringOwned,
    {
        unsafe {
            for child in children {
                let mut item = QStandardItem::new();
                let txt = child.to_qstring();
                item.set_text(&txt);
                item.set_editable(false);
                // now we set a single child
                if add_empty_gchild == true {
                    let mut child_item = QStandardItem::new();
                    child_item.set_text(&qs(""));
                    child_item.set_editable(false);
                    item.append_row_q_standard_item(child_item.into_ptr());
                }
                parent.append_row_q_standard_item(item.into_ptr());
            }
        }
    }
}
