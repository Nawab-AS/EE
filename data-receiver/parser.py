import re

def parse(log_file: str):
    import csv
    with open(log_file, "r") as file:
        lines = "".join(file.readlines()[2:])

    blocks = {}
    for match in re.finditer(r"=== ECC (\d+) / RSA \d+ bits ===\n(.*?)(?:\n\n|\Z)", lines, re.DOTALL):
        trials = []
        bit_size = int(match.group(1))
        block_content = match.group(2).strip()

        for trial in block_content.split("\n"):
            trial = trial.strip()
            data = re.match(r"Trial #\d{1,3}: ECC = (\d+), RSA = (\d+), ECC fails = (\d+), RSA fails = (\d+)", trial)
            if not data:
                print(f"Error parsing trial: {bit_size} bits, Trial #{len(trials) + 1}")
                trials.append({
                    "ECC": None,
                    "RSA": None,
                    "ECC fails": None,
                    "RSA fails": None
                })
                continue

            trials.append({
                "ECC": int(data.group(1)),
                "RSA": int(data.group(2)),
                "ECC fails": int(data.group(3)),
                "RSA fails": int(data.group(4))
            })

        blocks[bit_size] = trials

    if not blocks:
        print("No valid result blocks found.")
        return

    # [Algorithm, Trial #, <8 bits time>, <8 bits #errors>, <16 bits time>, <16 bits #errors>, ...]
    bit_sizes = sorted(blocks.keys())
    num_trials = max(len(trials) for trials in blocks.values())

    with open("results.csv", "w", newline="") as csvfile:
        writer = csv.writer(csvfile)

        header = ["Algorithm", "Trial #"]
        for bit_size in bit_sizes:
            header.append(f"{bit_size} bits time")
        writer.writerow(header)

        def value_or_dash(value):
            return "-" if value is None or value == "" else value

        for algorithm, time_key in [
            ("RSA", "RSA"),
            ("ECC", "ECC"),
        ]:
            for trial_idx in range(num_trials):
                row = [algorithm, trial_idx + 1]
                for bit_size in bit_sizes:
                    trials = blocks.get(bit_size, [])
                    if trial_idx < len(trials):
                        trial = trials[trial_idx]
                        time_value = value_or_dash(trial.get(time_key))
                    else:
                        time_value = "-"

                    row.append(time_value)

                writer.writerow(row)

    print("CSV file 'results.csv' written.")

if __name__ == "__main__":
    parse("./data.log")