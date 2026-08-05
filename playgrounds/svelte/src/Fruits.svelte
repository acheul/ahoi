<script lang="ts">
    import { useReadHail, useTell } from "./bridge";
    import type { Fruit } from "../../ahoi-wasm-tsrs/bindings/Fruit";

    const lastFruit = useReadHail("LastFruit"); // enum on the wire
    const fruitCounts = useReadHail("FruitCounts"); // JS Map on the wire
    const tell = useTell();

    const label = (fruit: Fruit | undefined) =>
        fruit === undefined
            ? "-"
            : typeof fruit === "string"
              ? fruit
              : `Banana(${fruit.Banana})`;
</script>

<section>
    <h3>Fruits</h3>
    <p>
        last: <b id="last-fruit">{label($lastFruit)}</b> · counts:
        <b id="fruit-counts">
            {#each [...$fruitCounts] as [name, n] (name)}
                <span>{`${name}×${n} `}</span>
            {/each}
        </b>
    </p>
    <button id="set-apple" onclick={() => tell({ SetFruit: "Apple" })}>Apple</button>
    <button id="set-banana" onclick={() => tell({ SetFruit: { Banana: "yellow" } })}>
        Banana
    </button>
</section>
