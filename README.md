# [Automatic Spoon](https://tene.github.io/automatic-spoon/)

Web App for choosing random items from lists you define.  The name was chosen randomly by GitHub's random name generator.

In the unlikely even that anyone besides me ever uses this, please feel free to file an issue with any requests you have.  I just threw together the simplest thing that could possibly work and stopped there, but I'm open to requests for functionality, UI, and style requests.

The lists live entirely in your browser's local storage.  This does not connect to any server anywhere, and no list data leaves your computer except through any links you click on.

## Use and Development

You can use whatever release of [Automatic Spoon](https://tene.github.io/automatic-spoon/) I'm personally using in your browser.  This may change unpredictably, and might lose any data you've stored there.  Save the export if you care about data here.  If I ever get around to adding support for loading exported data, I'll try to support loading old exports if it's convenient.

Build with `yarn run build`

Serve locally with `yarn run start:dev`

This app is built in Rust using [Yew](https://yew.rs/).  As GitHub mentions, I started this project using yewstack/yew-wasm-pack-template

## TODO

- Choose order and rearrange lists within a group
- Turn off heartbeat timer while all lists in chosen group are frozen.