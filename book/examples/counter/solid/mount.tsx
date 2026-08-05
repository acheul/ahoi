/**
 * Wraps the shown component in its pier. Not displayed in the book.
 *
 * Lives behind the lazy boundary in `island.tsx`, because importing
 * `setup/solid/bridge` runs a top-level `await wasmInit()`.
 */
import { PierProvider } from "../../setup/solid/bridge";
import Counter from "./Counter";

export default function Mount() {
    return (
        <PierProvider pier="Top">
            <Counter />
        </PierProvider>
    );
}
