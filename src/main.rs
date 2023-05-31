#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::*;
use std::fs;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use open;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(egui::vec2(640.0, 760.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Media Renaming Tool",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    // files and paths
    picked_path: Option<String>,
    picked_file: Option<String>,
    files_in_picked_path: Vec<String>,
    new_path: Option <String>,
    new_file_name: String,
    
    // text fields
    author: String,
    series: String,
    episode: String,
    title: String,
    extension: String,
    
    // toggles
    toggle_help: bool,

    // errors
    was_rename_successful: bool,
    error_msg: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            // files and paths
            picked_path: None,
            picked_file: None,
            files_in_picked_path: Default::default(),
            new_path: None,
            new_file_name: Default::default(),

            // text fields
            author: Default::default(),
            series: Default::default(),
            episode: Default::default(),
            title: Default::default(),
            extension: Default::default(),
            
            // toggles
            toggle_help: false,

            // errors
            was_rename_successful: false,
            error_msg: Default::default(),
        }
    }
}

impl MyApp {
    fn move_file(&self) -> Result<(), String> {
        if let (Some(picked_path), Some(picked_file), Some(new_path)) =
            (&self.picked_path, &self.picked_file, &self.new_path)
        {
            let source_path = format!("{}/{}", picked_path, picked_file);
            let destination_path = format!("{}/{}", new_path, self.new_file_name);

            // Move the file
            match fs::rename(&source_path, &destination_path) {
                Ok(()) => Ok(()),
                Err(err) => Err(format!("Failed to move the file: {}", err)),
            }
        } else {
            Err(String::from("Invalid paths or file names provided."))
        }
    }

    fn display_filenames_as_buttons(&mut self, ui: &mut Ui) {
        let mut sorted_files_in_picked_path = self.files_in_picked_path.clone();
        sorted_files_in_picked_path.sort();  // Sort the files_in_picked_path alphabetically

        for filename in &sorted_files_in_picked_path {
            if ui.button(filename).clicked() {
                self.picked_file = Some(filename.to_string());

                // Clear the old value of the extension
                self.extension.clear();

                // Set the new extension
                let file_extension = self.picked_file
                    .as_ref()
                    .and_then(|file| file.split('.').last())
                    .map(|extension| extension.to_string());

                if let Some(extension) = file_extension {
                    self.extension.push('.');
                    self.extension.push_str(&extension);
                }
            }
        }
    }

    fn some_method(&mut self, path: Option<String>) {
        match path {
            Some(path) => {
                let folder_path = path.clone(); // Clone the path before passing it to fs::read_dir
    
                self.picked_path = Some(path.clone());
                self.new_path = Some(path);
    
                // Clear the files_in_picked_path vector before populating it with the files of the newly selected folder
                self.files_in_picked_path.clear();
    
                // Read the files in the folder, excluding folders
                if let Ok(entries) = fs::read_dir(&folder_path) {
                    self.files_in_picked_path.extend(entries.filter_map(|entry| {
                        entry.ok().and_then(|entry| {
                            let path = entry.path();
                            path.file_name()
                                .and_then(|file_name| file_name.to_str())
                                .filter(|_| path.is_file())
                                .map(|file_name| file_name.to_owned())
                        })
                    }));
                }
            }
            None => {
                // Handle the case when no folder is selected
                self.picked_path = None;
                self.new_path = None;
                self.files_in_picked_path.clear();
            }
        }
    }
    
    
}



impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //
        // top
        //
        egui::TopBottomPanel::top("my_top").show(ctx, |ui| {
            ui.heading("Media Renaming Tool");
        });

        //
        // center
        //
        egui::CentralPanel::default().show(ctx, |ui| {
            // display text of the current file
            
            ui.add_space(5.0);
            ui.horizontal_wrapped(|ui| {
                if let Some(mut picked_file) = self.picked_file.clone() {   
                    ui.text_edit_singleline(&mut picked_file);
                }
                if ui.button("Open File...").clicked() {
                    open_file(self.picked_path.clone(), self.picked_file.clone());
                }
            });


            // where we display the copy buttons
            ui.add_space(15.0);
            ui.horizontal_wrapped(|ui| {
                handle_button(ui, "[", "[");
                handle_button(ui, "]", "]");
                handle_button(ui, "(", "(");
                handle_button(ui, ")", ")");
                handle_button(ui, "{", "{");
                handle_button(ui, "}", "}");
                            
                let parts = split_file_name(self.picked_file.clone());

                for part in &parts {
                    let cloned_part = part.clone();  // Clone the part
                    
                    if ui.button(part).clicked() {
                        ui.output_mut(|o| o.copied_text = cloned_part);
                    }
                }
            });


            // display the grid of text boxes
            ui.add_space(15.0);
            ui.horizontal_wrapped(|ui| {                
                egui::Grid::new("bottom_grid")           
                .min_col_width(7.0)
                .max_col_width(ui.available_width() * 0.50)
                .show(ui, |ui| {
                    handle_field(ui, "Author: ", &mut self.author);
                    handle_field(ui, "Series: ", &mut self.series);
                    handle_field(ui, "Episode: ", &mut self.episode);
                    handle_field(ui, "Title: ", &mut self.title);
                    handle_field(ui, "Extension: ", &mut self.extension);
                });
                ui.end_row();
            }); 

            // takes in the textboxes input to figure out a final name
            self.new_file_name = [&self.author,&self.series,&self.episode,&self.title,&self.extension]
                .iter()
                .filter(|&&field| !field.is_empty())
                .map(|&field| field.clone())
                .collect::<Vec<String>>()
                .join("");


            // display name preview and save directory
            ui.add_space(15.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Name Preview:").size(12.0));
                ui.label(RichText::new(&self.new_file_name).size(12.0).color(Color32::from_rgb(110, 255, 110)));
                ui.end_row();
                ui.label(RichText::new("Save Directory").size(10.0));
                if let Some(new_path) = &self.new_path {
                    ui.label(RichText::new(new_path).size(10.0));
                }    
                
            });
            
            
            // action buttons
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                // open Directory
                if ui.button("Open Directory").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.picked_path = Some(path.display().to_string());
                        self.new_path = Some(path.display().to_string());
                        
                        self.some_method(self.picked_path.clone());
                    }
                }
                
                // write directory
                if ui.button("Write Directory").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.new_path = Some(path.display().to_string());
                    }
                }

                // Write file button
                if ui.button(RichText::new("Rename File").color(Color32::from_rgb(227, 118, 118))).clicked() {
                    let result = self.move_file();
                
                    match result {
                        Ok(()) => {
                            println!("Rename Successful!");
                            self.error_msg = "Rename Successful!".to_string();
                            self.was_rename_successful = true;
                            self.picked_file = None;
                            self.new_file_name = Default::default();
                            self.author = Default::default();
                            self.series = Default::default();
                            self.episode = Default::default();
                            self.title = Default::default();
                            self.extension = Default::default();
                            self.some_method(self.picked_path.clone());
                        }
                        Err(error) => {
                            println!("Error: {}", error);
                            self.error_msg = error;
                            self.was_rename_successful = false;
                        }
                    }
                }

                // help
                if ui.button("Help").clicked() {
                    self.toggle_help = !self.toggle_help;
                }
            });
            
            ui.add_space(10.0);


            ui.label(self.error_msg.clone());
            
            if self.toggle_help {
                // info text
                ui.label(RichText::new("Usage Information").size(10.0));
                ui.label(RichText::new("- Open Folder = Pick the directoy that contains the files you wish to rename").size(10.0));
                ui.label(RichText::new("- Write Directory = Leave it to keep renamed file in the same directory, other wise choose where to put the renamed file").size(10.0));
                ui.label(RichText::new("- Rename File= Execute command to rename file").size(10.0));
                ui.label(RichText::new("- Clicking the buttons that contains words, or barckets, will coppy that text to the clipboard.").size(10.0));
                ui.label(RichText::new("- Right-Click the text boxes to paste text.").size(10.0));
                ui.label(RichText::new("- On the right hand side of the text boxes, you can add spaces, hyphens, and commas").size(10.0));
            }
            ui.end_row();
            ui.add_space(15.0);
        });


        //
        // bottom
        //
        TopBottomPanel::bottom("bottom").resizable(true).show(ctx, |ui| {
            ScrollArea::vertical()
                .min_scrolled_height(325.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.display_filenames_as_buttons(ui);                 
                });

        });
    }
}





fn split_file_name(file_name: Option<String>) -> Vec<String> {
    let separators: Vec<&str> = vec![",", ".", "-", "[", "]", "{", "}", "(", ")"];

    file_name
        .unwrap_or_default()
        .split(|c: char| separators.contains(&c.to_string().as_str()))
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(String::from)
        .collect()
}


// Helper function to handle clipboard paste and space addition with label
fn handle_field(ui: &mut egui::Ui, label: &str, field: &mut String) {
    ui.label(label);
    
    if ui.text_edit_singleline(field).secondary_clicked() {
        if let Ok(clipboard_content) = ClipboardContext::new().and_then(|mut ctx| ctx.get_contents()) {
            field.push_str(&clipboard_content);
        }
    }
    
    if ui.button("   ").clicked() {
        field.push(' ');
    }
    if ui.button(" - ").clicked() {
        field.push_str(" - ");
    }
    if ui.button(", ").clicked() {
        field.push_str(", ");
    }
    
    ui.end_row();
}

// Helper function to handle button click and assign value to copied_text
fn handle_button(ui: &mut egui::Ui, label: &str, value: &str) {
    if ui.button(label).clicked() {
        ui.output_mut(|o| o.copied_text = value.to_string());
    }
}


fn open_file(picked_path: Option<String>, picked_file: Option<String>) {
    let mut file_path = String::new();

    if let Some(path) = picked_path {
        file_path.push_str(&path);
        if !file_path.ends_with('/') {
            file_path.push('/');
        }
    }

    if let Some(file) = picked_file {
        file_path.push_str(&file);
    }

    if let Err(err) = open::that(&file_path) {
        eprintln!("Error opening file: {}", err);
    }
}