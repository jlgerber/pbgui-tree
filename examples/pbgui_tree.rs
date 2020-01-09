use packybara::packrat::PackratDb;
use packybara::packrat::{Client, NoTls};
use pbgui_tree::tree;
use qt_core::QResource;
use qt_widgets::{QApplication, QFrame, QMainWindow, QPushButton, QWidget};
use rustqt_utils::{create_vlayout, qs};

pub struct ClientProxy {}

impl ClientProxy {
    pub fn connect() -> Result<Client, Box<dyn std::error::Error>> {
        let client = Client::connect(
            "host=127.0.0.1 user=postgres dbname=packrat password=example port=5432",
            NoTls,
        )?;
        Ok(client)
    }
}

fn main() {
    QApplication::init(|_app| unsafe {
        let _result = QResource::register_resource_q_string(&qs("/Users/jgerber/bin/pbgui.rcc"));
        let mut main_window = QMainWindow::new_0a();
        let mut main_widget = QFrame::new_0a();
        let main_widget_ptr = main_widget.as_mut_ptr();

        // main_layout
        let mut main_layout = create_vlayout();
        let mut main_layout_ptr = main_layout.as_mut_ptr();
        main_widget.set_layout(main_layout.into_ptr());
        // set main_widget as the central widget in main_window
        main_window.set_central_widget(main_widget.into_ptr());

        let b1 = QPushButton::from_q_string(&qs("top"));
        let b2 = QPushButton::from_q_string(&qs("Bottom"));
        main_layout_ptr.add_widget(b1.into_ptr());
        let mut mytree = tree::DistributionTreeView::create(main_widget_ptr);
        mytree.set_default_stylesheet();
        mytree.set_packages(vec!["foo", "bar", "bla"]);

        mytree.clear_packages();
        //mytree.set_packages(vec!["maya", "nuke", "houdini"]);
        //mytree.add_package("mari");
        main_layout_ptr.add_widget(b2.into_ptr());

        //tb.set_default_stylesheet();
        let client = ClientProxy::connect().expect("Unable to connect via ClientProxy");
        let mut db = PackratDb::new(client);

        let results = db
            .find_all_packages()
            .query()
            .expect("unable to find_all_packages");
        let results = results.iter().map(|s| s.name.as_str()).collect::<Vec<_>>();
        //tb.set_level_items(results);
        mytree.set_packages(results);
        main_window.show();
        QApplication::exec()
    });
}
