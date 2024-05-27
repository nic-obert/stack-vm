#!/bin/sh

echo $'\nRust:'
find . -type f -name "*.rs" ! -wholename "**/target/*" | xargs wc -l | tail -n 1

echo $'\nAssembler:'
find assembler -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nVM:'
find vm -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\nVMlib:'
find vmlib -type f -name "*.rs" | xargs wc -l | sort -nr

echo $'\n\nAssembly:'
find . -type f -name "*.asm" | xargs wc -l | sort -nr

echo 
