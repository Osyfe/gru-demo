//no console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod root;

fn main()
{
    root::start();
}
