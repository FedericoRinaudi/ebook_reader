use druid::{Widget, WidgetExt};
use druid::widget::{Button, Click, ControllerHost, DisabledIf};
use crate::ApplicationState;

pub struct Buttons {}

impl Buttons {
    pub fn btn_next() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new(">").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.current_book.go_on(1);
            data.update_view()
        })
    }

    pub fn btn_prev() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("<").on_click(|_ctx, data: &mut ApplicationState, _env| {
            data.current_book.go_back(1);
            data.update_view()
        })
    }

    pub fn btn_confirm() -> DisabledIf<ApplicationState, ControllerHost<Button<ApplicationState>, Click<ApplicationState>>> {
        Button::new("Confirm")
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /* EDIT MODE -> EDIT MODE, CONFIRM CHANGES */
                // data.current_book.save();
                data.xml_backup = data.current_book.chapters[data.current_book.get_nav().get_ch()].xml.clone();
                data.modified.insert(data.current_book.get_nav().get_ch()); /* Inserisco se non è già presente il capitolo corrente in quelli modificati */
            }).disabled_if(|data: &ApplicationState, _| data.view.current_view.len()==0 || data.view.current_view[0].is_err())
    }

    pub fn btn_edit() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("Edit")
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /*  VIEW MODE -> EDIT MODE */
                data.xml_backup = data.current_book.chapters[data.current_book.get_nav().get_ch()].xml.clone();
                data.view.set_window_size_view(<(f64, f64)>::from(ctx.window().get_size()));
                ctx.window().set_size(data.view.get_window_size_edit());
                ctx.window().set_title("EDIT MODE");
                data.edit = !data.edit
            })
    }


    pub fn bnt_view() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("View")
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /*  EDIT MODE -> VIEW MODE */
                data.view.set_window_size_edit(<(f64, f64)>::from(ctx.window().get_size()));
                ctx.window().set_size(data.view.get_window_size_view());
                ctx.window().set_title("VIEW MODE");
                data.edit = !data.edit;
            })
    }

    pub fn bnt_discard() -> ControllerHost<Button<ApplicationState>, Click<ApplicationState>> {
        Button::new("Discard")
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /* EDIT MODE -> EDIT MODE, Discard Changes */
                data.current_book.update_xml(data.xml_backup.clone());
                data.update_view();
            })
    }

    //TODO: Button to save on file
    pub fn btn_save() -> DisabledIf<ApplicationState, ControllerHost<Button<ApplicationState>, Click<ApplicationState>>> {
        Button::new("Save on File")
            .on_click(|ctx, data: &mut ApplicationState, _env| {
                /* SAVE CHANGES ON NEW FILE */
                data.current_book.save(data.modified.clone());
                // data.modified.clear(); TODO: Apri nuovo file scritto ogni volta che si va a salvare
            }).disabled_if(|data: &ApplicationState, _| data.view.current_view.len()==0 || data.view.current_view[0].is_err())
    }

}