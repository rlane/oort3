- Made rescoring more robust to panics in the simulation.

### 0.83.1 - 2024-12-01

- Fixed rescoring not removing asterisk when the time is unchanged.

### 0.83.0 - 2024-12-01

- turn() oscillates less around the target speed.
- The leaderboard now shows an asterisk if the solution hasn't been validated yet.

### 0.82.0 - 2024-11-30

- Added a second radar to cruisers.
- Added a `select_radar` API.

### 0.81.0 - 2024-11-29

- Increased enemy distance in tutorial_search.
- Added comments to challenge mission initial code.

### 0.80.0 - 2024-11-08

- Added the ability to link the editor to multiple files or a directory.
- Changed the link-to-file shortcut to Ctrl-Y.

### 0.79.3 - 2024-04-18

- Fixed crashes related to the welcome window.

### 0.79.2 - 2024-04-13

- Fixed leaderboard bug for usernames containing invalid characters.
- Fixed leaderboard entries with empty code.

### 0.79.1 - 2023-12-27

- Changed rendering to "snap back" to a snapshot without any interpolation when the game is paused.

### 0.79.0 - 2023-12-24

- Added cruiser_defense scenario.

### 0.78.2 - 2023-11-21

- Fixed using multiple radios with send_bytes/receive_bytes.

### 0.78.1 - 2023-11-17

- Fixed parsing of heading command in the sandbox scenario.
- Updated documentation for cruiser main gun.

### 0.78.0 - 2023-11-16

- Added a favicon (by hipparcos).
- Converted the cruiser flak gun into a long-range heavy cannon.
- Increased size of frigate and cruiser.
- Added an early sandbox scenario.
- Fixed interaction of radar distance filter with edge returns.
- Disabled edge returns for small ships as an optimization.

### 0.77.0 - 2023-11-13

- Added hotkeys for restarting simulation to canvas (by twof).
- Moved user code to the root of the crate so that the `crate` keyword works as expected.
- Added CPU usage to the debug text.

### 0.76.3 - 2023-11-09

- Fixed a couple of bugs in the race scenario.

### 0.76.2 - 2023-11-08

- Fixed acceleration debug line rotation.

### 0.76.1 - 2023-11-07

- Fixed radar with 90 degree beamwidth.

### 0.76.0 - 2023-11-07

- Added an Asteroid Race scenario (by t-fi).
- Moved ship models so that the center of mass is close to the ship's position.
- Changed radar to return a contact when the beam intersects a bounding circle.
- Changed the C support to C++ including the standard library.
- Increased the memory limit per ship to 2 MB.
- Fixed acceleration debug line when boost is active.
- Fixed fighter lateral flare rendering.

### 0.75.0 - 2023-10-30

- Integrated wasm-submemory to remove reliance on Rust's memory safety.
- Added very early C language support.
- Added a check-solution-reliability binary.

### 0.74.1 - 2023-10-25

- Fixed chase camera when ship is destroyed (by 0e4ef622)

### 0.74.0 - 2023-10-25

- Optimized shape drawing functions (by mwlsk).
- Added times to battle binary output (by ByteRanger).
- Added chase camera keybinding 'c' (by 0e4ef622)
- Replaced std floating point functions with nalgebra versions for determinism.

### 0.73.0 - 2023-10-18

- Set radar minimum width to TAU/720 for fighters, missiles, and torpedos.
- Randomized seeds for tournaments.
- Added a progress bar to the tournament binary.
- Switched leaderboard links to use shortcodes for easy URL copying.
- Replaced the leaderboard's copy button with a link to play against that AI.
- Removed smoking asteroid from welcome scenario.
- Fixed incorrect info and typos in the API docs.

### 0.72.0 - 2023-10-17

- Added average time to versions tab (by ByteRanger).

### 0.71.0 - 2023-10-16

- Optimized polygon drawing (by ByteRanger).
- Changed ship error handling to leave the ship alive with its AI disabled.
- Fixed scrolling of tournament results.
- Fixed log message for out of gas errors.
- Fixed comments for radar distance filter APIs.
- Removed tournament button from planetary_defense.

### 0.70.0 - 2023-10-15

- Enabled enhanced-determinism feature in Rapier.

### 0.69.0 - 2023-10-15

- Created a nightly job to rescore the leaderboards.

### 0.68.2 - 2023-10-15

- Allowed default attribute in sanitizer.
- Fixed building tools on ARM.
- Fixed tools use of shortcode service.

### 0.68.1 - 2023-10-15

- Fixed numbering of radio and search tutorials.
- Changed compiler service to return more appropriate HTTP status codes.
- Allowed repr, inline, and must_use attributes in sanitizer.
- Removed nonexistent vector cross product from quick reference.

### 0.68.0 - 2023-10-14

- Added keybinding for slow motion.
- Changed fighter minimum radar width to match docs.

### 0.67.4 - 2023-10-14

- Fixed typo of math_rs in docs.
- Fixed leaderboard header colspan.
- Fixed docs for current_tick().
- Added more logging to services.

### 0.67.3 - 2023-10-14

- Fixed a leaderboard sorting bug.
- Fixed that usernames couldn't contain capital letters.

### 0.67.2 - 2023-10-13

- Added caching to leaderboard service.

### 0.67.1 - 2023-10-13

- Fixed leaderboard service sending Discord messages for ranks > 10.

### 0.67.0 - 2023-10-13

- Fixed ship physics body sleeping.
- Changed leaderboard to always display player and those immediately above or below them.

### 0.66.1 - 2023-10-13

- Fixed API docs for set_radar_heading.
- Fixed loading code from leaderboard when linked to a file.
- Removed ability to accelerate in the gunnery scenario.

### 0.66.0 - 2023-10-12

- Added a new tutorial_lead scenario between rotation and deflection.
- Changed scenario time display to millisecond precision.
- Fixed docs for maximum radar width.
- Increased the world size in the orbit scenario.
- Randomized starting positions in the orbit scenario.

### 0.65.1 - 2023-10-02

- Fixed panic handler for String payloads (by @Easyoakland)
- Added population counter to planetary_defense and increase difficulty.

### 0.65.0 - 2023-09-20

- Added a second radio to fighters.
- Added a mini_fleet scenario.
- Changed tournament scenarios to randomly swap starting positions.
- Made oorandom available to player code.

### 0.64.3 - 2023-09-18

- Fixed a nondeterminism issue in the radar code.

### 0.64.2 - 2023-09-15

- Made toggles for blur/nlips/debug persistent.
- Increased max NLIPS ghost size.
- Improved initial zoom calculation to more precisely fit ships on screen.
- Removed primitive_duel from the scenario list.
- Removed the heading debug line.

### 0.64.1 - 2023-09-14

- Panic messages from ships are now logged to the JS console.

### 0.64.0 - 2023-09-12

- Changed missiles to not explode when running out of fuel.
- Deprecated the ShapedCharge ability and effectively made it the default.
- Increased range of missile and torpedo explosions.

### 0.63.0 - 2023-09-08

- Revamped ability API to allow multiple abilities and deactivation (by @Easyoakland)
- Added active abilities to picked ship stats.
- Added a play button to leaderboard rows.

### 0.62.0 - 2023-09-08

- Changed seed selection links to preserve player and opponent code.
- Added a "seed" tab so you can change the seed without editing the URL.
- Added the seed to the debug status line.
- Removed the upload-shortcode button when using encrypted code.
- Fixed an NLIPS-related panic in planetary_defense.

### 0.61.1 - 2023-09-08

- Fixed a rendering bug with NLIPS.
- Removed the FPS display when paused, since the game enters power-save mode.

### 0.61.0 - 2023-09-07

- Added an NLIPS rendering mode toggled with 'v' that makes ships more visible when zoomed out.

### 0.60.0 - 2023-09-07

- Added `send_bytes` and `receive_bytes` radio functions (by @Nudelmeister).
- Made the byteorder crate available to user code.
- Implemented link-to-file editing for Firefox.
- Changed the UI to drop frames when rendering at <60fps so that the simulation runs in real time.

### 0.59.0 - 2023-09-06

- Made the maths_rs crate available to player code and switched Vec2 to be a type alias.
- Adding a missing `radar_ecm_mode` accessor.
- Reenabled caching for web worker files.

### 0.58.0 - 2023-09-04

- Added a replay-paused button (by @Nudelmeister).

### 0.57.3 - 2023-09-04

- Improved sanitizer error message.
- Increased limit for debug lines and documented it.

### 0.57.2 - 2023-09-03

- Enabled submitting to tournament if solution won any simulations.
- Fixed radio message docs.
- Fixed a panic in WASM crash error handling.

### 0.57.1 - 2023-09-02

- Added time to the victory status line.

### 0.57.0 - 2023-09-02

- Improved error messages when player code panics or hits the instruction limit.
- Added an `rgb` helper function.
- Reduced aliasing of debug lines (including the radar beam).
- Fixed clipping of fighter lateral engine flare.
- Added a "replay" button that reuses the same seed and doesn't trigger the "Mission Complete" screen.

### 0.56.0 - 2023-09-02

- Enabled Vimium link hints for tabs.
- Turned off automatic debug line display in the welcome scenario.

### 0.55.1 - 2023-09-01

- Improved documentation.

### 0.55.0 - 2023-09-01

- Made f64 methods visible to rust-analyzer.
- Added blur to scenario lines.
- Decreased text contrast.
- Limited attributes supported in player code.

### 0.54.2 - 2023-08-01

- Improved blur performance on iOS / Mac.
- Reduced scale of early tutorials.

### 0.54.1 - 2023-07-21

- Added blur effect.
- Fixed game speed on high refresh rate monitors.

### 0.54.0

- Added engine flares.

### 0.53.0

- Tweaked team placement and composition in tournament scenarios.
- Increased missile acceleration and decreased delta-v.
- Changed missiles to explode on running out of fuel.
- Increased frigate main gun bullet TTL.
- Added scenario_name(), world_size(), and id() APIs.
- Added the ability to load code versions from other scenarios.
- Decreased minimum radar beamwidth and increased maximum.
- Replaced the reference AI with a more readable one.
- Reverted limited turret slew rate.
- scan() API now works in scenarios without radar.

### 0.52.0

- Added RSSI and SNR to radar scan result.
- Randomized radar return and noise.
- Added weapon reload_ticks API.
- Removed missile_duel and furball scenarios and added squadrons scenario.
- Fixed a bug in the welcome scenario.

### 0.51.0

- Added a simple version control system.
- Tweaked planetary_defense scenario.
- Increased world size in fleet scenario.
- Added orbit scenario.
- Made planets block radar.

### 0.50.0

- Increased missile and torpedo lifetime.
- Added limited fuel for missiles and torpedos.
- Added APIs to get current health and fuel.
- Limited slew rate for turrets.
- Increased world size for frigate and cruiser duel scenarios.

### 0.49.0

- Ships that collide with the edge of the world will be destroyed.

### 0.48.0

- Increased bullet lifetime.

### 0.47.0

- Optimized radar calculations.
- Fixed accelerate documentation.
- Added radar_duel scenario.

### 0.46.2

- Added vector examples to early tutorials.
- Added Vec2::splat.
- Derived PartialEq and Default for Vec2.

### 0.46.1

- Fixed selecting ships on high DPI devices.

### 0.46.0

- Fixed radar to fall off with the 4th power of distance.
- Added ECM (noise jamming).
- Made tutorial_squadron easier.

### 0.45.1

- Made radar lines more visible.

### 0.45.0

- Fixed dragging causing ships to be deselected.
- Fixed background grid misalignment.
- Increased default world size to 40 km and made it configurable.
- Increased size of frigate and cruiser.
- Limited maximum radar beamwidth to 22.5 degrees.
- Clamped returned contacts inside radar beam.
- Added probability of detecting contacts outside reliable radar range.

### 0.44.0

- Updated to Rust 1.68.0.
- Added support for deploying staging environments.
- Simplified services and deployment.
- Added a tournament subcommand usable without database access.

### 0.43.2

- Show mission complete screen when opponent code is modified.

### 0.43.1

- Updated to Rust 1.67.1.
- Fixed tournament determinism.

### 0.43.0

- Added a tournament results page.

### 0.42.2

- Technical improvements for tournament submissions.

### 0.42.1

- Fixed welcome window not being displayed.

### 0.42.0

- Tweaked primitive_duel scenario and made it visible.
- Added shortcodes for tournament submissions.

### 0.41.0

- Reordered tutorials.
- Added a radio tutorial.
- Added missiles to the squadron tutorial and increased range.
- Changed search tutorial to use one ship per team.

### 0.40.0

- Ramp up missile spawning over time in planetary_defense scenario.
- Added vector cross product to API.
- Fixed format code action.
- Renamed tutorial scenarios.

### 0.39.2

- Fixed collisions in planetary_defense scenario.

### 0.39.1

- Fixed resizing editor window.
- Increased time limit in tutorial09.

### 0.39.0

- Decreased enemy health in tutorial09.
- Reduced CPU usage when simulation is paused.

### 0.38.2

- Further improved simulation performance.
- Fixed initial headings in the belt scenario.

### 0.38.1

- Added a serve binary to run a development server.
- Improved simulation performance.

### 0.38.0

- Moved enemies further away in tutorial08.
- Fixed a crash when rendering empty text.

### 0.37.0
- Added a button to the mission complete screen to copy a shortcode for the player's AI.

### 0.36.1
- Updated dependencies.
- Added validation for text and debug line APIs.

### 0.36.0
- Created a compiler library for use in tools.
- Fixed an issue where the RNG was shared between ships.

### 0.35.0

- Added support for fetching leaderboard entries via URL parameters.

### 0.34.2

- Sped up release process.

### 0.34.0

- Added more Discord integrations.

### 0.33.0

- Randomize starting location in tutorial03 and tutorial04 to make tutorial05 easier.

### 0.32.1

- Send a Discord message when a new version is released.

### 0.32.0

- Removed missiles from tutorial07.
- Started a changelog.
