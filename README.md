# Zhur

## What is Zhur?

Zhur [ʐur], named after [the soup](https://en.wikipedia.org/wiki/West_Slavic_fermented_cereal_soups#Poland), is a platform for writing simple, small-scale HTTP-driven web apps with WebAssembly, utilizing Kevin Hoffman's [waPC protocol](https://github.com/wapc). It was the subject of my bachelor's degree dissertation in university. Zhur's features include a built-in key/value store and a user SDK that facilitates returning various types of HTTP responses.

## Design

Zhur is designed as several modules running in separate processes, communicating over Unix domain sockets. The most important ones are:

- a core process, responsible for hosting the user-submitted WebAssembly code and running the applications in response to incoming requests
- a gateway process, running as an HTTP server that forwards its requests to the core process
- a database process, responsible for storing application data in a sled key/value database.
