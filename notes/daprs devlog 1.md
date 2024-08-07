- Writing a successor to PAPR
	- I understand Rust and optimized programming much better now since working on my game engine
	- New version will be much more lightweight and easier to use
- Design goals
	- Fast; focus on `Copy` types and shared references wherever possible
	- No allocations on the real-time audio thread
	- Precompute as much as possible
	- Type safety wherever possible
- 3 API levels:
	- Low-level `processor` API where raw access to audio data are available for arbitrary processing in Rust
	- Mid-level `builder` API for constructing signal processor graphs intuitively
		- Potential future bindings to other languages e.g. Python
	- High-level `functional` API to provide macro-like functionality on top of the `builder` API
- 