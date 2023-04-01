import katex from "katex";

export default function renderLatex(inputHTML: string): string {
    // Match everything that is in between $;
    
    // $...$ means inline latex
    // $$...$$ Means centered on a new line
    
    const regex = /\$\$?[^<>]*?\$\$?/gmd;
    const regexResult = inputHTML.matchAll(regex);
    
    const originalHTMLChunks = inputHTML.split(regex);

    console.log("Original chunks", originalHTMLChunks);

    let newHTML = "";

    let i = 0;
    while (true) {
        const currentResult = regexResult.next().value;
        if (!currentResult) {
            break;
        }

        const currentMatch = currentResult[0];

        // Differentiate between $ and $$
        const isDouble = currentMatch[0] == "$" && currentMatch[1] == "$";
        const trimLength = isDouble ? 2 : 1;

        console.log(originalHTMLChunks[i], currentMatch);

        newHTML += originalHTMLChunks[i] + katex.renderToString(currentMatch.substring(trimLength, currentMatch.length-trimLength), { trust: false, output: "mathml", displayMode: isDouble });
        
        i++;
    }

    newHTML += originalHTMLChunks[originalHTMLChunks.length-1];

    return newHTML;
}


