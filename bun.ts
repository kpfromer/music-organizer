#!/usr/bin/env bun

import { $ } from "bun";
import { readdir, readFile, writeFile } from "fs/promises";
import { join } from "path";

async function findRustFiles(dir: string): Promise<string[]> {
  const files: string[] = [];
  const entries = await readdir(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await findRustFiles(fullPath)));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(fullPath);
    }
  }

  return files;
}

async function main() {
  const srcDir = "src";
  const allFiles = await findRustFiles(srcDir);
  
  // Find files with log::
  const filesWithLog = [];
  for (const file of allFiles) {
    const content = await readFile(file, "utf-8");
    if (content.includes("log::")) {
      filesWithLog.push(file);
    }
  }

  if (filesWithLog.length === 0) {
    console.log("No files found with log::");
    return;
  }

  console.log("Found files with log:::");
  filesWithLog.forEach(f => console.log(f));
  console.log("");

  // Replace log:: with tracing::
  for (const file of filesWithLog) {
    console.log(`Processing ${file}...`);
    let content = await readFile(file, "utf-8");
    content = content.replace(/log::/g, "tracing::");
    await writeFile(file, content, "utf-8");
  }

  console.log("Replaced log:: with tracing::");
  console.log("");

  // Remove log imports and add tracing imports
  for (const file of filesWithLog) {
    console.log(`Updating imports in ${file}...`);
    let content = await readFile(file, "utf-8");
    
    // Remove standalone use log; lines
    content = content.replace(/^use log;$/gm, "");
    content = content.replace(/^use log::\*;$/gm, "");
    
    // Remove log from multi-import lines
    content = content.replace(/use log, /g, "use ");
    content = content.replace(/, log,/g, ",");
    content = content.replace(/, log;/g, ";");
    
    // Add tracing import if file uses tracing:: and doesn't have it
    if (content.includes("tracing::") && !content.includes("use tracing")) {
      const lines = content.split("\n");
      let insertIndex = -1;
      
      // Find first use statement
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].trim().startsWith("use ")) {
          insertIndex = i;
          break;
        }
      }
      
      if (insertIndex !== -1) {
        // Insert before first use
        lines.splice(insertIndex, 0, "use tracing;");
      } else {
        // Find last mod declaration
        let modIndex = -1;
        for (let i = 0; i < lines.length; i++) {
          if (lines[i].trim().startsWith("mod ") || lines[i].trim().startsWith("pub mod ")) {
            modIndex = i;
          }
        }
        
        if (modIndex !== -1) {
          // Insert after last mod
          lines.splice(modIndex + 1, 0, "use tracing;");
        } else {
          // Insert at top
          lines.unshift("use tracing;");
        }
      }
      
      content = lines.join("\n");
    }
    
    await writeFile(file, content, "utf-8");
  }

  console.log("Done!");
  console.log("");
  console.log("Run 'cargo fmt' to format the code");
}

main().catch(console.error);

