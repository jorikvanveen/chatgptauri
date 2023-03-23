<script lang="ts">
    import Input from "./lib/Input.svelte";

    let apiKey = "";
</script>

<h1>Settings</h1>

<Input label="API Key" password bind:value={apiKey} />


