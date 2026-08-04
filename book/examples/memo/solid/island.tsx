/**
 * The island MDX imports. Not displayed in the book.
 *
 * `client:only` skips server *rendering*, but the MDX import is still hoisted
 * and evaluated during prerender. Loading the real component lazily keeps wasm
 * out of that pass, and out of the initial page bundle.
 */
import { Suspense, lazy } from "solid-js";

const Mount = lazy(() => import("./mount"));

export default function Island() {
    return (
        <Suspense fallback={<p>loading wasm…</p>}>
            <Mount />
        </Suspense>
    );
}
