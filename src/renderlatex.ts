import { assert } from "console";
import katex from "katex";

interface Range {
	start: number,
	end: number
}

function findLatexBlocks(inputHTML: string): Range[] {
	let ranges: Range[] = []
	let i = 0;

	let isInSmallCodeBlock = false;
	let isInBigCodeBlock = false;
	let isInSmallLatexBlock = false;
	let isInBigLatexBlock = false;
	
	while (i < inputHTML.length) {
		const character = inputHTML[i];

		let isEscaped = inputHTML[i-1] == "\\";

		if (character == "`" && !isEscaped) {
			let isBigCodeBlockDelimiter = inputHTML[i+1] == "`" && inputHTML[i+2] == "`";

			if (isBigCodeBlockDelimiter) {
				isInBigCodeBlock = !isInBigCodeBlock;
				i = i + 3;
				continue;
			}

			if (!isInBigCodeBlock) {
				// This is a small code block delimiter
				isInSmallCodeBlock = !isInSmallCodeBlock;
			}
		}

		if (character == "$" && !isInBigCodeBlock && !isInSmallCodeBlock && !isEscaped) {
			let isBigLatexDelimiter = inputHTML[i+1] == "`"

			if (isBigLatexDelimiter) {
				if (isInBigLatexBlock) {
					isInBigLatexBlock = false;
					console.log(ranges[ranges.length-1])
					ranges[ranges.length-1].end = i+1;
					i = i + 2;
					continue;
				}

				isInBigLatexBlock = true;
				ranges.push({ start: i, end: -1 })
				i = i + 2;
				continue;
			}

			if (isInBigLatexBlock) {
				i++;
				continue;
			}
			
			if (isInSmallLatexBlock) {
				isInSmallLatexBlock = false;
				console.log(ranges[ranges.length-1])
				ranges[ranges.length-1].end = i;
			} else {
				isInSmallLatexBlock = true;
				ranges.push({ start: i, end: -1 })
			}
		}

		i++;
	}

	return ranges;
}

function getChunksInRanges(html: string, ranges: Range[]): string[] {
	const chunks: string[] = [];

	for (const range of ranges) {
		chunks.push(html.substring(range.start, range.end + 1))
	}

	return chunks
}

function getChunksOutsideRanges(html: string, ranges: Range[]) {
	const chunks: string[] = [];

	for (let i = 0; i < ranges.length; i++) {
		const currentRange = ranges[i];
		const lastEnd = i == 0 ? -1 : ranges[i-1].end;

		chunks.push(html.substring(lastEnd+1, currentRange.start))
		// Get all the html between the last end and the current start	
	}

	chunks.push(html.substring((ranges[ranges.length-1] || { end: -1 }).end+1, html.length))

	return chunks;
}

export default function renderLatex(inputHTML: string): string {
    // Match everything that is in between $;
    
    // $...$ means inline latex
    // $$...$$ Means centered on a new line
    
    //const regex = /\$\$?[^<>]*?\$\$?/gmd;
    //const regex = /\$\$?[^]*?\$\$?/gmd;
    //const regexResult = inputHTML.matchAll(regex);
   	
	const ranges = findLatexBlocks(inputHTML);
	const latexBlocks = getChunksInRanges(inputHTML, ranges);
    const originalHTMLChunks = getChunksOutsideRanges(inputHTML, ranges);

    console.log("Original chunks", originalHTMLChunks);

    let newHTML = "";

    let i = 0;
    while (true) {
        const currentResult = latexBlocks[i];
        if (!currentResult) {
            break;
        }

        // Differentiate between $ and $$
        const isDouble = currentResult[0] == "$" && currentResult[1] == "$";
        const trimLength = isDouble ? 2 : 1;

        console.log(originalHTMLChunks[i], currentResult);

        let renderedChunk = currentResult.substring(trimLength, currentResult.length-trimLength);

        try {
            renderedChunk = katex.renderToString(currentResult.substring(trimLength, currentResult.length-trimLength), { trust: false, output: "mathml", displayMode: isDouble });
        } catch(e) {
            console.warn("Failed to render TeX", e)
        }

        newHTML += originalHTMLChunks[i] + renderedChunk;
        
        i++;
    }

    newHTML += originalHTMLChunks[originalHTMLChunks.length-1];

    return newHTML;
}


