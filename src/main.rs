mod spotifyclient;
mod volume_control;

use rspotify::model::track::FullTrack;
use tokio::time::{delay_for, Duration};

const AD_LENGTH: Duration = Duration::from_secs(30); // default ad length

#[tokio::main]
async fn main() {
    println!("Starting Silentify...");
    let spotify = &spotifyclient::build_spotify_instance().await;
    let (name, email) = spotifyclient::get_current_displayname(spotify).await;
    println!("The current logged in user is {} ({})", name, email);

    loop {
        let curr_track = spotifyclient::get_curr_playing_track(spotify).await;

        if curr_track.is_some() {
            let playing = curr_track.unwrap();

            match playing.item {
                Some(full_track) => {
                    // something is playing - restore the volume now
                    println!("Some track is playing. Restoring volume...");
                    volume_control::restore_vol();

                    let curr_progress = match playing.progress_ms {
                        Some(p) => Duration::from_millis(p.into()),
                        None => Duration::from_millis(0), // going to assume that if progress result is empty but track info exists, nothing has started playing yet but will play
                    };

                    print_curr_playing(&full_track, curr_progress);

                    // one thing to consider is network lag, processing when calculating the progress of the song
                    let remaining_duration =
                        Duration::from_millis(full_track.duration_ms.into()) - curr_progress;

                    println!(
                        "Checking again in {} seconds...",
                        Duration::as_secs(&remaining_duration)
                    );
                    wait_for(remaining_duration).await;
                }
                // in rspotify library None means the item is not a track, album, playist, episode, etc... so it must be an ad if something is playing
                None => {
                    println!("An AD is playing. Muting for {} seconds...", Duration::as_secs(&AD_LENGTH));
                    volume_control::mute_volume();
                    wait_for(AD_LENGTH).await;
                }
            }
        } else {
            println!("Nothing is currently playing. Check that Spotify is playing something. Exiting...");
            break;
        }
    }
}

async fn wait_for(duration: Duration) {
    delay_for(duration).await;
    println!("Waited for {} seconds", duration.as_secs());
}

fn print_curr_playing(full_track: &FullTrack, progress_ms: Duration) {
    println!(
        "Currently playing {} is {} by {} lasting for {} seconds with {} seconds already played.",
        full_track._type.as_str(),
        full_track.name,
        spotifyclient::format_artist_vec(&full_track.artists),
        Duration::as_secs(&Duration::from_millis(full_track.duration_ms.into())),
        progress_ms.as_secs()
    );
}
