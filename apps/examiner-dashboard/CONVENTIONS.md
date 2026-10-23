# Coding conventions

## Testing

Individual React components should use the renderer from `testing-utils/render.tsx`, since they are not wrapped in any providers.

Views, in contrast, should use the renderer from `@testing-library/react`, since they're already wrapped.
