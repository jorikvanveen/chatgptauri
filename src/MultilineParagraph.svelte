<script lang="ts">
    export let text: string;

    $: parts = text.split("\n");
</script>

{#each parts as part, i}
    {part}
    {#if i != parts.length - 1}
        <br/>
    {/if}
{/each}
