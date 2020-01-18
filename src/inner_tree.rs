use qt_core::{QAbstractItemModel, QModelIndex, QSize, QString, SlotOfQString, WidgetAttribute};
use qt_gui::q_icon::{Mode, State};
use qt_gui::QIcon;
use qt_gui::{QStandardItem, QStandardItemModel};
use qt_widgets::{
    cpp_core::{CastInto, CppBox, DynamicCast, MutPtr, Ref, StaticUpcast},
    q_abstract_item_view::EditTrigger,
    q_header_view::ResizeMode,
    QComboBox, QFrame, QLabel, QLayout, QLineEdit, QPushButton, QTreeView, QWidget,
};
use rustqt_utils::{create_hlayout, create_vlayout, qs, set_stylesheet_from_str, ToQStringOwned};

const STYLE_STR: &'static str = include_str!("../resources/tree.qss");

/// A struct holding the QTreeView and providing a simple Api, mirrored
/// by the parent.
pub struct InnerTreeView<'a> {
    pub parent_frame: MutPtr<QFrame>,
    pub cbox: MutPtr<QComboBox>,
    pub filter_cb: MutPtr<QPushButton>,
    pub filter_frame: MutPtr<QFrame>,
    pub filter: MutPtr<QLineEdit>,
    pub view: MutPtr<QTreeView>,
    pub filter_slot: SlotOfQString<'a>,
}

impl<'a> InnerTreeView<'a> {
    /// create an InnerTreeView instance. This inner tree allows us
    /// to use the tree's api in Slots exposed by the parent
    pub fn create<T>(parent_widget: MutPtr<T>) -> InnerTreeView<'a>
    where
        T: StaticUpcast<QWidget>,
    {
        unsafe {
            let mut qframe = QFrame::new_0a();
            let qframe_ptr = qframe.as_mut_ptr();
            let mut layout = create_vlayout();
            let mut layout_ptr = layout.as_mut_ptr();
            qframe.set_layout(layout.into_ptr());

            let parent_widget = parent_widget.static_upcast_mut();
            parent_widget.layout().add_widget(qframe.into_ptr());

            let (cbox_p, filter_btn) = Self::create_cbox(layout_ptr);

            let mut treeview = QTreeView::new_0a();
            treeview.set_object_name(&qs("PackageTreeView"));
            let mut treeview_ptr = treeview.as_mut_ptr();
            let mut filter_frame = Self::new_qframe();
            let mut filter_frame_ptr = filter_frame.as_mut_ptr();
            let filter = Self::new_filter(filter_frame_ptr);
            layout_ptr.add_widget(filter_frame.into_ptr());

            filter_frame_ptr.set_visible(false);

            treeview_ptr.set_edit_triggers(EditTrigger::NoEditTriggers.into());
            treeview_ptr.set_root_is_decorated(true);
            treeview_ptr.set_items_expandable(true);
            treeview_ptr.set_uniform_row_heights(true);
            treeview_ptr.set_header_hidden(true);
            parent_widget.layout().add_widget(treeview.into_ptr());

            let mut model = QStandardItemModel::new_0a();
            model.set_column_count(2);
            let model_ptr = model.as_mut_ptr();
            treeview_ptr.set_model(model.into_ptr());
            treeview_ptr.header().resize_section(1, 20);
            treeview_ptr.header().set_stretch_last_section(false);
            treeview_ptr
                .header()
                .set_section_resize_mode_2a(0, ResizeMode::Stretch);

            let itv = InnerTreeView {
                parent_frame: qframe_ptr,
                cbox: cbox_p,
                filter_cb: filter_btn,
                filter_frame: filter_frame_ptr,
                filter,
                view: treeview_ptr.clone(),
                filter_slot: SlotOfQString::new(move |new_str: Ref<QString>| {
                    let root = QModelIndex::new();
                    if new_str.to_std_string() == "" {
                        for cnt in (0..model_ptr.row_count_0a()).rev() {
                            treeview_ptr.set_row_hidden(cnt, root.as_ref(), false)
                        }
                    } else {
                        for cnt in (0..model_ptr.row_count_0a()).rev() {
                            let item = model_ptr.item_2a(cnt, 0);
                            let txt = item.text();
                            if txt.contains_q_string(new_str) {
                                treeview_ptr.set_row_hidden(cnt, root.as_ref(), false)
                            } else {
                                treeview_ptr.set_row_hidden(cnt, root.as_ref(), true)
                            }
                        }
                    }
                }),
            };

            itv.filter.text_changed().connect(&itv.filter_slot);

            itv
        }
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

    /// Retrieve a mutable pointer to the combobox
    pub fn combobox(&self) -> MutPtr<QComboBox> {
        self.cbox
    }
    /// set the row as hidden
    pub unsafe fn set_row_hidden(&self, idx: Ref<QModelIndex>, hidden: bool) {
        let mut view = self.view;
        view.set_row_hidden(0, idx, hidden);
    }
    /// Given a type that implements ToQstringOwned, append a distribution
    pub fn add_package<T: ToQStringOwned>(&self, input: T) {
        unsafe {
            let mut model = self.model();
            let icon = QIcon::from_q_string(&QString::from_std_str(":/images/package_md.png"));
            let row_count = model.row_count_0a();
            let mut parent = model.invisible_root_item();
            let mut item = QStandardItem::new();
            item.set_text(&input.to_qstring());
            item.set_icon(&icon);
            item.set_editable(false);
            parent.append_row_q_standard_item(item.into_ptr());
            model.set_row_count(row_count + 1);
        }
    }

    /// Clear the package list from the model
    pub fn clear_packages(&self) {
        unsafe {
            let mut model = self.model();
            for c in (0..model.row_count_0a()).rev() {
                model.clear_item_data(&self.model().index_2a(c, 0));
            }
            model.set_row_count(0)
        }
    }

    /// Given a vector of a type that implements ToQstringOwned, append a distribution
    pub fn set_packages<T: ToQStringOwned>(&self, inputs: Vec<T>) {
        unsafe {
            let mut model = self.model();
            let mut parent = model.invisible_root_item();
            //model.clear(); // this removes columns as well. and segfaults
            let row_cnt = inputs.len() as i32;
            //
            let icon = QIcon::from_q_string(&QString::from_std_str(":/images/package_md.png"));
            for input in inputs {
                let mut item = QStandardItem::new();
                let txt = input.to_qstring();
                item.set_text(&txt);
                item.set_icon(&icon);
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
    pub fn add_child<I>(&self, parent: MutPtr<qt_gui::QStandardItem>, child: I)
    where
        I: ToQStringOwned,
    {
        unsafe {
            let mut item = QStandardItem::new();
            let txt = child.to_qstring();
            item.set_text(&txt);
            item.set_editable(false);
            let mut parent = parent;
            parent.append_row_q_standard_item(item.into_ptr());
        }
    }

    /// Remove all sites from the combobox
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns None
    pub fn remove_sites(&self) {
        unsafe {
            self.combobox().clear();
        }
    }
    /// Set combobox sites, replacing any extant sites
    ///
    /// # Arguments
    /// * `items` - Vector of items
    ///
    /// # Returns
    /// * None
    pub fn set_sites<'c, I>(&self, items: Vec<I>, current: I)
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
                self.combobox().add_item_q_string(&qs(item.as_ref()));
                cnt += 1;
            }
            self.combobox().set_current_index(idx);
        }
    }

    /// set childred
    pub fn set_children<I>(
        &self,
        mut parent: MutPtr<qt_gui::QStandardItem>,
        children: Vec<I>,
        add_empty_gchild: bool,
    ) where
        I: ToQStringOwned,
    {
        unsafe {
            let mut cnt = 0;
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
                let mut icon_item = QStandardItem::new();
                icon_item.set_editable(false);
                parent.append_row_q_standard_item(item.into_ptr());
                parent.set_child_3a(cnt, 1, icon_item.into_ptr());
                cnt += 1;
            }
        }
    }

    pub fn clear_selection(&self) {
        unsafe {
            self.view.selection_model().clear_selection();
        }
    }
    /// Retrieve the filter combobox
    pub fn filter_cb(&self) -> MutPtr<QPushButton> {
        self.filter_cb
    }

    /// turn visibility of frame off and on
    pub fn set_filter_visibility(&self, visible: bool) {
        unsafe {
            let mut filter_frame = self.filter_frame;
            filter_frame.set_visible(visible);
        }
    }

    unsafe fn new_qframe() -> CppBox<QFrame> {
        let mut qf = QFrame::new_0a();
        qf.set_object_name(&qs("PackageFilterFrame"));
        let layout = create_hlayout();
        qf.set_layout(layout.into_ptr());
        qf
    }

    unsafe fn new_filter(parent: MutPtr<QFrame>) -> MutPtr<QLineEdit> {
        let label = QLabel::from_q_string(&qs("Package Filter"));
        parent.layout().add_widget(label.into_ptr());
        let mut qle = QLineEdit::new();
        qle.set_attribute_2a(WidgetAttribute::WAMacShowFocusRect, false);
        qle.set_object_name(&qs("PackageFilter"));
        let qle_ptr = qle.as_mut_ptr();
        parent.layout().add_widget(qle.into_ptr());
        qle_ptr
    }

    /// Set the stylesheet to the internal stylesheet
    ///
    /// # Arguments
    /// * None
    ///
    /// # Returns
    /// *None
    pub fn set_default_stylesheet(&self) {
        set_stylesheet_from_str(STYLE_STR, self.parent_frame);
    }

    /// Change the max number of items displayed in the combobox's dropdown
    /// list
    ///
    /// # Arguments
    /// * `max` - Maximum number of visible items in the comobobox's dropdown
    ///
    /// # Returns
    /// * None
    pub fn set_cb_max_visible_items(&self, max: i32) {
        unsafe {
            self.combobox().set_max_visible_items(max);
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
            cbox.set_object_name(&qs("SiteComboBox"));
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
