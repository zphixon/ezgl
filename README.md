# ez gl

Major refactorings in-progress in the [Glutin](https://github.com/rust-windowing/glutin) project are underway to decouple it from [Winit](https://github.com/rust-windowing/winit). While this is a big win for those who want fine-grained control over how they get their GL context set up, it adds some complexity for others who don't particularly care (that is, myself).

This library aims to reduce the friction between the user and sweet, sweet GL calls, via [glow](https://github.com/grovesNL/glow). Here's how:

1. Create your window that implements `HasRawWindowHandle` and `HasRawDisplayHandle`
2. Create an `Ezgl` instance using your window
3. Call GL functions on it

Todo:

- Increase support
  - [ ] Android
  - [ ] iOS
  - [ ] Web
