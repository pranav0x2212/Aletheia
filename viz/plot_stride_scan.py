#!/usr/bin/env python3
"""
Aletheia Stride Scan Visualization (Improved)

Better visual clarity:
- Log-scale x-axis (base 2)
- Cache-line boundary highlighted
- Cleaner styling
- Stronger annotations
"""

import json
import pandas as pd
import matplotlib.pyplot as plt
from pathlib import Path
import sys


def load_results(filepath):
    results = []
    with open(filepath, 'r') as f:
        for line in f:
            if line.strip():
                results.append(json.loads(line))
    return pd.DataFrame(results)


def plot_stride_scan(df, output_dir):
    stride_data = df[df['experiment'] == 'stride_scan'].copy()

    if stride_data.empty:
        print("No stride_scan data found")
        return

    stride_data = stride_data.sort_values('stride')

    fig, ax = plt.subplots(figsize=(11, 6.5))
    ax2 = ax.twinx()

    # --- Plot CPU ---
    cpu = stride_data[stride_data['mode'] == 'cpu']
    if not cpu.empty:
        ax.plot(
            cpu['stride'],
            cpu['runtime_ms'],
            marker='o',
            linewidth=2.5,
            markersize=7,
            label='CPU Runtime',
        )

    # --- Plot Memory Engine ---
    mem = stride_data[stride_data['mode'] == 'memory_engine']
    if not mem.empty:
        ax.plot(
            mem['stride'],
            mem['runtime_ms'],
            marker='s',
            linewidth=2.5,
            markersize=7,
            label='Memory Engine Runtime',
        )

        ax2.plot(
            mem['stride'],
            (mem['cache_misses'] / mem['cache_references']) * 100,
            linestyle='--',
            marker='^',
            color='tab:green',
            linewidth=2,
            markersize=7,
            label='Memory Engine Cache Miss Rate',
        )

    # --- Log scale (IMPORTANT) ---
    ax.set_xscale('log', base=2)

    # --- Labels ---
    ax.set_xlabel('Stride (bytes)', fontsize=12)
    ax.set_ylabel('Runtime (ms)', fontsize=12)
    ax2.set_ylabel('Cache Miss Rate (%)', fontsize=12)
    ax2.set_ylim(0, 60)
    ax.set_title(
        'Runtime and Cache Miss Rate vs Memory Access Stride',
        fontsize=14,
        pad=12
    )

    # --- Clean grid ---
    ax.grid(True, linestyle='--', alpha=0.2)

    # --- X ticks (powers of 2 only) ---
    strides = sorted(stride_data['stride'].unique())
    ax.set_xticks(strides)
    ax.get_xaxis().set_major_formatter(plt.ScalarFormatter())

    # --- Highlight cache line ---
    cache_line = 64
    ax.axvline(x=cache_line, linestyle='--', alpha=0.6)

    ax.text(
        cache_line,
        ax.get_ylim()[1] * 0.85,
        'Cache line (~64B)',
        rotation=90,
        verticalalignment='center',
        horizontalalignment='right',
        fontsize=10,
        alpha=0.8
    )

    # --- Legend ---
    lines1, labels1 = ax.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax.legend(lines1 + lines2, labels1 + labels2, frameon=False)

    plt.tight_layout()

    output_path = Path(output_dir) / 'stride_scan.png'
    output_path.parent.mkdir(parents=True, exist_ok=True)

    plt.savefig(output_path, dpi=300)
    print(f"✓ Saved: {output_path}")

    plt.close()


def main():
    # Get the project root (parent of viz/)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    results_file = project_root / 'results' / 'rpi-results' / 'stride_scan.jsonl'
    output_dir = script_dir / 'output'

    if not results_file.exists():
        print(f"Missing: {results_file}")
        sys.exit(1)

    df = load_results(results_file)
    plot_stride_scan(df, output_dir)


if __name__ == '__main__':
    main()
