//Edited
use std::{io};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::{executor, theme, Application, Command, Element, Font, Length, Settings, Theme};
use iced::widget::{button, column, container, horizontal_space, pick_list, row, text, text_editor, tooltip};
use iced::highlighter::{self,Highlighter};

fn main() -> iced::Result{
    Editor::run(Settings{
        fonts: vec![include_bytes!("../fonts/edtior-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    })
}

#[derive(Debug,Clone)]
enum Message{
    Edit(text_editor::Action),
    Open, 
    New,
    Save,
    FileSaved (Result<PathBuf,Error>),
    FileOpened(Result<(PathBuf,Arc<String>), Error>),
    ThemeSelected(highlighter::Theme)
}

struct Editor{
    is_edited : bool,
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
    theme: highlighter::Theme
}

#[derive(Debug,Clone)]
enum Error{
    DialogClosed,
    IOFailed(io::ErrorKind)
}
impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default ;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>){
        (
            Self{
                is_edited: true,
                path:None,
                content: text_editor::Content::new(),
                error:None,
                theme: highlighter::Theme::SolarizedDark
            }, 
            Command::perform(
                load_file(default_file()),
                Message::FileOpened ),
        )
        
    }

    fn title(&self) -> String {
        String::from("A cool editor!")
    }

    fn update(&mut self, message: Message) -> Command<Message>{
        match message{
            Message::Edit(action) => {
                self.is_edited = self.is_edited || action.is_edit();
                self.content.edit(action);
                self.error = None;
                Command::none()
            } 

            Message::Open => {
                Command::perform(pick_file(), Message::FileOpened)
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                Command::none()
            }
            Message::FileOpened(Ok((path,content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);
                self.is_edited= false;
                Command::none()
            }

            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                self.is_edited = false;
                Command::none()
            }

            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }

            Message::Save => {
                let text= self.content.text();

                Command::perform(save_file(self.path.clone(),text),Message::FileSaved)
            }

            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }

            Message::ThemeSelected(theme) => {
                self.theme = theme; 
                Command::none()
            }
        }

    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action(save_icon(),"Save",self.is_edited.then_some(Message::Save)),
            action(new_icon(),"New File", Some(Message::New)),
            action(open_icon(),"Open File",Some(Message::Open)),
            horizontal_space(Length::Fill),
            pick_list(highlighter::Theme::ALL, Some(self.theme), Message::ThemeSelected)].spacing(10);

        let input = text_editor(&self.content)
            .on_edit(Message::Edit)
            .highlight::<Highlighter>(highlighter::Settings {
                theme: self.theme,
                extension: self.path
                    .as_ref()
                    .and_then(|path| path.extension()?.to_str())
                    .unwrap_or("rs")
                    .to_string(),
            }, |highlight, _theme|{
                highlight.to_format()
            });

        let status_bar = {

            let status= if let Some(Error::IOFailed(error)) = self.error.as_ref() {
                text(error.to_string())
            }
            else{
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) =>  text(path).size(14),
                    None => text("New File")
                }
            };

            let position = {
                let(line,column) = self.content.cursor_position();
                text(format!("{}:{}", line+1,column+1))
            };

            row![status,horizontal_space(Length::Fill), position]
        
        };

        container(column![controls,input,status_bar].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark(){
            Theme::Dark
        }else{
            Theme::Light
        }
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf,Arc<String>), Error>{
   let contents = tokio::fs::read_to_string(&path)
       .await
       .map(Arc::new)
       .map_err(|error| error.kind())
       .map_err(Error::IOFailed)?;
    
   Ok((path,contents))
}

async fn pick_file() -> Result<(PathBuf,Arc<String>), Error>{
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file..")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;
    load_file(handle.path().to_owned()).await
}

fn default_file() -> PathBuf{
    PathBuf::from(format!("{}/src/main.rs",env!("CARGO_MANIFEST_DIR")))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf,Error> {
    let path = if let Some(path) = path {
        path
    }else{
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };
    
    tokio::fs::write(&path, text)
        .await
        .map_err(|error| Error::IOFailed(error.kind()))?;
    
    Ok(path)
}

fn action<'a>(content: Element<'a, Message>, label: &str, on_press:Option<Message>) -> Element<'a, Message>{
    let is_disabled = on_press.is_none();
    
    tooltip(button(container(content).width(30).center_x())
                .on_press_maybe(on_press)
                .padding([5,10])
                .style(if is_disabled{
                    theme::Button::Secondary
                }else{
                    theme::Button::Primary
                }),
            label, 
            tooltip::Position::FollowCursor)
            .style(theme::Container::Box)
        .into()

}
fn new_icon<'a>() -> Element<'a, Message>{
    icon_helper('\u{E800}')
}

fn open_icon<'a>() -> Element<'a,Message>{
    icon_helper('\u{F115}')
}

fn save_icon<'a>() -> Element<'a, Message>{
    icon_helper('\u{E801}')
}

fn icon_helper<'a,Message>(codepoint: char) -> Element<'a,Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");
    text(codepoint).font(ICON_FONT).into()
}  
