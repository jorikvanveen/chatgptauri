<script lang="ts">
    export let label: string;
    export let disabled: boolean = false;
</script>

<button {disabled} on:click>{label}</button>

<style>
    button {
        transition: .2s;
        font-size: 1rem;
        padding: .5rem;
        background: var(--teal);
        border: none;
        outline: none;
        border-radius: 7px;
        color: var(--bg);
        cursor: pointer;
    }

    button:hover {
        box-shadow: 2px 2px 10px 0px #000000;
    }

    button:disabled {
        opacity: 0.5;
    }
</style>
