use crate::prelude::*;

/// System which applies all queued audio track commands
pub(crate) fn apply_audio_track_commands(ctx: &mut SystemsContext, _: Query<()>) {
    // TODO: currently only the main track is supported
    let mut track = ctx.resources.get_mut::<AudioTrack>().expect("AudioTrack resource not found"); 
    track.apply(ctx.resources); 
}
