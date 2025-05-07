import subprocess
import tempfile
import os

# RST snippets extracted from the Rust tests in src/main.rs
import re

def extract_rst_snippets_from_rust(file_path):
    snippets = []
    try:
        with open(file_path, 'r') as f:
            content = f.read()

        # Regex to find test functions and potentially associated raw strings
        # This is a simplified approach and might need refinement
        # It looks for #[test] fn <name> and then the first r#""# or r"" after it
        test_pattern = re.compile(r'#\[test\]\s+fn\s+(\w+)\s*\(\)\s*\{.*?\}', re.DOTALL)
        raw_string_pattern = re.compile(r'r#"(.*?)"#|r"(.*?)"', re.DOTALL)

        for match in test_pattern.finditer(content):
            test_name = match.group(1)
            test_body = match.group(0) # The full match of the test function block

            # Search for raw string literals within the test body
            raw_string_match = raw_string_pattern.search(test_body)
            if raw_string_match:
                # Group 1 is for r#""#, Group 2 is for r""
                snippet_content = raw_string_match.group(1) if raw_string_match.group(1) is not None else raw_string_match.group(2)
                snippets.append((test_name, snippet_content))
            else:
                 print(f"Warning: No raw string literal found in test '{test_name}'")


    except FileNotFoundError:
        print(f"Error: File not found at {file_path}")
    return snippets

def validate_snippet(test_name, rst_content):
    print(f"--- Validating: {test_name} ---")
    # rstcheck can be sensitive to leading/trailing whitespace for the whole document.
    # Let's strip the overall snippet if it's just whitespace, but preserve internal structure.
    rst_content_to_check = rst_content.strip()

    if not rst_content_to_check:
        print("Skipping empty snippet (rstcheck would likely error or warn).")
        print("Result: SKIPPED (Empty)")
        print("-" * 30)
        return

    try:
        with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".rst") as tmp_file:
            tmp_file.write(rst_content_to_check)
            tmp_file_path = tmp_file.name
        
        # Using --report-level INFO to get more feedback, can be changed to WARNING or ERROR
        # For some snippets, rstcheck might issue INFO or WARNING for things that are not strictly errors
        # but are stylistic issues or minor problems.
        # We are primarily interested if it's "syntactically valid" enough for a directive parser.
        # `rstcheck` exits with 0 on success (no errors/warnings at or above report level), 
        # 1 for errors/warnings, and >1 for other issues.
        # Allow custom directives by name
        # Ignore custom directives by name
        # Ignore custom directives by name
        custom_directive_names = "mydirective,my_directive_name,first_directive,second_directive,first,second,existing_directive,mydirective-extra,my-directive_name"
        result = subprocess.run(["rstcheck", "--report-level", "INFO", "--ignore-directives", custom_directive_names, tmp_file_path], capture_output=True, text=True)
        
        if result.returncode == 0:
            print("Result: VALID")
            if result.stdout:
                print("rstcheck output (stdout):\n", result.stdout)
        else:
            print(f"Result: INVALID (rstcheck exit code: {result.returncode})")
            if result.stdout:
                print("rstcheck output (stdout):\n", result.stdout)
            if result.stderr:
                print("rstcheck error output (stderr):\n", result.stderr)

    except FileNotFoundError:
        print("ERROR: rstcheck command not found. Make sure it's installed and in your PATH.")
    except Exception as e:
        print(f"An error occurred: {e}")
    finally:
        if 'tmp_file_path' in locals() and os.path.exists(tmp_file_path):
            os.remove(tmp_file_path)
    print("-" * 30)

if __name__ == "__main__":
    rust_file_path = "src/main.rs" # Path to your Rust file
    RST_SNIPPETS = extract_rst_snippets_from_rust(rust_file_path)

    total_snippets = len(RST_SNIPPETS)
    print(f"Starting validation of {total_snippets} RST snippets extracted from {rust_file_path} using rstcheck...\n")
    for name, snippet in RST_SNIPPETS:
        validate_snippet(name, snippet)
    print("\nValidation complete.")