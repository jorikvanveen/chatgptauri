<script lang="ts">
    export let value: string;
    export let label: string;
    export let password: boolean = false;
    $: inputType = password ? "password" : "text"

    let id = Math.floor(Math.random() * 0xffffffff).toString()
</script>

<label for={id}>{label}</label><br/>
{#if password}
    <input {id} type="password" bind:value={value} />
{:else}
    <input {id} type="text" bind:value={value} />
{/if}

<style>
    input {
        background: var(--dark-blue);
        color: var(--fg);
        border: none;
        outline: none;
        box-sizing: border-box;
        padding: .5rem;
        border-radius: 7px;
        width: 100%;
        font-size: 1rem;
    }

</style>
