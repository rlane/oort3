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
