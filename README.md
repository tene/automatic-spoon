# [Automatic Spoon](https://tene.github.io/automatic-spoon/)

Web App for choosing random items from lists you define.  The name was chosen randomly by GitHub's random name generator.

In the unlikely even that anyone besides me ever uses this, please feel free to file an issue with any requests you have.  I just threw together the simplest thing that could possibly work and stopped there, but I'm open to requests for functionality, UI, and style requests.

The lists live entirely in your browser's local storage.  This does not connect to any server anywhere, and no list data leaves your computer except through any links you click on.

## Use and Development

You can use whatever release of [Automatic Spoon](https://tene.github.io/automatic-spoon/) I'm personally using in your browser.  This may change unpredictably, and might lose any data you've stored there.  I plan to add some kind of export functionality so you can save whatever you've made with this, and I'll try to support importing old exports whenever it's easy and convenient.

Build with `yarn run build`

Serve locally with `yarn run start:dev`

This app is built in Rust using [Yew](https://yew.rs/).  As GitHub mentions, I started this project using yewstack/yew-wasm-pack-template

## TODO

- List Items with images, inks, and comments
- Choose order and rearrange lists within a group
- Flicker randomly through items in a list until the list is clicked on.