<div>
    <h1>Actinium Template</h1>
    <p><i>Full-stack development with Rust and Svelte</i></p>
</div>

- **Backend**: Rust with Actix-web for high-performance async API endpoints
- **Frontend**: Svelte, or any static compatible frontend framework, compiled to static assets (HTML, JS, CSS)
- **Build Process**: Frontend generates static files during build time, served alongside Rust API
- **Outbound Interface Layer**: A layer of indirection is implemented via a collection of *Drivers* that provide access to 
external resources through a standardized interface.
    - **Note:** We borrow the terminology 'Driver' because it sounds cool, the meaning is different in this context.
