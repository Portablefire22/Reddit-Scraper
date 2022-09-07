use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect(); // Gets the arguments from terminal.
    let username: String;
    if args.len() <= 1 {
        username = get_user();
    } else {
        username = args[1].clone();
    }
    println!("Searching for user: {}",username);
    get_profile(username);
}

fn get_user() -> String {
    println!("Username: ");
    let mut username = String::new();

    io::stdin().read_line(&mut username).expect("Failed to read line");
    
    let username: String = String::from(username.trim());
    username
}

fn get_profile(username: String) -> Result<(), Box<dyn std::error::Error>> {
    let profile_link = format!("https://reddit.com/user/{}/submitted.json",username);

    let profile_json = reqwest::blocking::get(profile_link)?.text()?;
    
    let json: serde_json::Value = serde_json::from_str(&profile_json)?;
    let json: &serde_json::Value = &json["data"]["children"];

    save_profile(json,&username)?;
    Ok(())
}   
// Send help
fn iterate_posts(json: &serde_json::Value, username: &String) -> Result<(), Box<dyn std::error::Error>>{
    let mut i = 0;
    std::fs::create_dir_all(format!("{}/posts",username))?;
    while json[i]["data"]["ups"].to_string() != "null"{  // A post will always have a "ups" value, thus "null" == no post
        let post_title: String;
        let mut post_body: String;
        let mut post_subreddit: String;
        println!("Title: {}\nUpvotes: {}",json[i]["data"]["title"],json[i]["data"]["ups"]);
        if json[i]["data"]["title"].to_string() != "null"{ // Video / Image posts do not have a title value, instead they have link_title
            post_title = json[i]["data"]["title"].to_string();
            post_body = json[i]["data"]["selftext"].to_string();
        } else { // Video / Image
            post_title = json[i]["data"]["link_title"].to_string();
            post_body = json[i]["data"]["body"].to_string();
        }
        post_subreddit = json[i]["data"]["subreddit"].to_string();
        let mut file_name: String;
        if json[i]["data"]["post_hint"].to_string() == "\"image\"" { // If the post is an image post
            file_name = format!("{}/posts/{}/{}.txt",username,post_title,post_title); 
            std::fs::create_dir_all(format!("{}/posts/{}",username,post_title))?; // Create directory for the post
            let mut img_url: String = json[i]["data"]["url"].to_string(); // Get the image url
            img_url.pop(); // Remove ending quotation marks
            img_url.remove(0); // Remove beginning quotation marks
            let img_bytes = reqwest::blocking::get(format!("{}",img_url))?.bytes()?; // Store the image into memory
            let image = image::load_from_memory(&img_bytes)?; // Load the image in a way the image crate can deal with it
            image.save(format!("{}/posts/{}/image.jpg",username,post_title))?; // Save the image
        }
        else if json[i]["data"]["post_hint"].to_string() == "\"hosted:video\""{ // If the post has a video
            file_name = format!("{}/posts/{}/{}.txt",username,post_title,post_title);
            std::fs::create_dir(format!("{}/posts/{}",username,post_title))?; // Create directory for post
            let mut video_url: String = json[i]["data"]["secure_media"]["reddit_video"]["fallback_url"].to_string(); // Get the video's url
            video_url.pop(); // Remove ending quotation marks
            video_url.remove(0); // Remove beginning quotation marks
            let video = reqwest::blocking::get(format!("{}",video_url))?.bytes()?; // Downloads the video's bytes
            let mut video_file = File::create(format!("{}/posts/{}/video.mp4",username,post_title))?; // Creates the file in memory
            dbg!(video_file.write_all(&video)?); // Writes and saves the file to disk
        } else {
            file_name = format!("{}/posts/{}.txt",username,post_title); // Just save a standalone file
        }
        let mut file = File::create(file_name)?; 
        let file_contents: String = format!("{}\nSubreddit: {}\nUpvotes: {}\nDownvotes: {}\n\n{}",post_title,post_subreddit,json[i]["data"]["ups"],json[i]["data"]["downs"],post_body);
        file.write_all(file_contents.as_bytes())?; // Write the post to disk
        i = i + 1;
    }
    Ok(())
}

fn save_profile(json: &serde_json::Value, username: &String) -> Result<(), Box<dyn std::error::Error>> {
    iterate_posts(json,username);
    Ok(())
}