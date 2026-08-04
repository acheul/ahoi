import { usePier } from "../../setup/solid/ahoi";

export default function ResourceDemo() {
    const sphere = usePier();
    const count = sphere.readHail("Count");
    const tenTimes = sphere.readHail("TenTimes"); // number | undefined
    const loading = sphere.readHail("TenTimesLoading");

    return (
        <div class="demo">
            <p>
                count: <b id="count">{count()}</b> · ×10 (async):{" "}
                <b id="ten-times">{tenTimes() ?? "—"}</b>{" "}
                <span id="loading">{loading() ? "(fetching…)" : ""}</span>
            </p>
            <button id="bump-1" onClick={() => sphere.tell({ Bump: 1 })}>
                +1
            </button>
        </div>
    );
}
