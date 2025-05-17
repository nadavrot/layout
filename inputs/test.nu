def --env main [
    ...args: string  # Capture any additional arguments
] {
    # Initialize an empty list for file paths
    mut all_files = []

    mut only_html = false
    for arg in $args {
        if ($arg | str contains --ignore-case 'only-html') {
            $only_html = true
        }
    }

    # Process each .dot file in inputs directory
    for file in (ls .\\inputs\\*.dot | get name) {
        # html flag
        if (($file | str contains --ignore-case 'html') == false) and $only_html {
            continue    
        }
        # First output file with random suffix
        let random_num = (random int 1..32767)
        let name1 = $"($env.TEMP)\\out_($random_num).svg"
        do {cargo run --bin layout $file -o $name1}
        $all_files = ($all_files | append $name1)

        # Second output file with random suffix
        let random_num = (random int 1..32767)
        let name2 = $"($env.TEMP)\\out_($random_num).svg"
        # do {dot -Tsvg $file -o $name2}
        # $all_files = ($all_files | append $name2)
    }

    # Open all generated files with the default browser
    if ((which firefox | length) > 0) {
        # If Firefox is installed, use it
        ^firefox $all_files
    } else {
        # Otherwise, use the default system browser
        for file in $all_files {
            do { start $file }
        }
    }
}
