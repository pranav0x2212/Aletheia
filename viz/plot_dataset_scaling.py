#!/usr/bin/env python3
"""
Aletheia Dataset Scaling Visualization Script

Reads JSONL experiment results and creates plots showing runtime scaling
across different dataset sizes for CPU vs memory_engine modes.
"""

import json
import pandas as pd
import matplotlib.pyplot as plt
from pathlib import Path
import sys


def load_results(filepath):
    """Load JSONL experiment results into a pandas DataFrame."""
    results = []
    try:
        with open(filepath, 'r') as f:
            for line in f:
                if line.strip():
                    results.append(json.loads(line))
        return pd.DataFrame(results)
    except FileNotFoundError:
        print(f"Error: Results file not found: {filepath}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in results file: {e}")
        sys.exit(1)


def plot_workload_scaling(df, workload_name, output_dir):
    """
    Plot runtime vs dataset size for a single workload.
    Shows both CPU and memory_engine modes on the same graph.
    """
    # Filter data for this workload
    workload_data = df[df['experiment'] == workload_name].copy()
    
    if workload_data.empty:
        print(f"Warning: No data found for workload '{workload_name}'")
        return
    
    # Sort by dataset size
    workload_data = workload_data.sort_values('dataset_size_mb')
    
    # Create figure and axis
    fig, ax = plt.subplots(figsize=(10, 6))
    
    # Plot CPU mode
    cpu_data = workload_data[workload_data['mode'] == 'cpu']
    ax.plot(
        cpu_data['dataset_size_mb'],
        cpu_data['runtime_ms'],
        marker='o',
        linewidth=2,
        markersize=8,
        label='CPU Mode',
        color='#FF6B6B'
    )
    
    # Plot memory_engine mode
    mem_data = workload_data[workload_data['mode'] == 'memory_engine']
    ax.plot(
        mem_data['dataset_size_mb'],
        mem_data['runtime_ms'],
        marker='s',
        linewidth=2,
        markersize=8,
        label='Memory Engine Mode',
        color='#4ECDC4'
    )
    
    # Configure axes and labels
    ax.set_xlabel('Dataset Size (MB)', fontsize=12, fontweight='bold')
    ax.set_ylabel('Runtime (ms)', fontsize=12, fontweight='bold')
    ax.set_title(f'{workload_name.replace("_", " ").title()} - Runtime Scaling', 
                 fontsize=14, fontweight='bold', pad=20)
    
    # Add grid
    ax.grid(True, alpha=0.3, linestyle='--')
    
    # Configure legend
    ax.legend(loc='upper left', fontsize=11, framealpha=0.95)
    
    # Format x-axis to show all dataset sizes
    ax.set_xticks(workload_data['dataset_size_mb'].unique())
    
    # Tight layout
    plt.tight_layout()
    
    # Save figure
    output_path = Path(output_dir) / f"{workload_name}_scaling.png"
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"✓ Saved: {output_path}")
    
    plt.close()


def plot_comparison(df, output_dir):
    """
    Create a side-by-side comparison of all workloads.
    """
    workloads = df['experiment'].unique()
    
    fig, axes = plt.subplots(1, len(workloads), figsize=(5 * len(workloads), 5))
    
    # Handle single workload case
    if len(workloads) == 1:
        axes = [axes]
    
    for idx, workload in enumerate(sorted(workloads)):
        ax = axes[idx]
        workload_data = df[df['experiment'] == workload].sort_values('dataset_size_mb')
        
        # Plot both modes
        cpu_data = workload_data[workload_data['mode'] == 'cpu']
        mem_data = workload_data[workload_data['mode'] == 'memory_engine']
        
        ax.plot(cpu_data['dataset_size_mb'], cpu_data['runtime_ms'], 
               marker='o', linewidth=2, markersize=8, label='CPU', color='#FF6B6B')
        ax.plot(mem_data['dataset_size_mb'], mem_data['runtime_ms'], 
               marker='s', linewidth=2, markersize=8, label='Memory Engine', color='#4ECDC4')
        
        ax.set_xlabel('Dataset Size (MB)', fontsize=11, fontweight='bold')
        ax.set_ylabel('Runtime (ms)', fontsize=11, fontweight='bold')
        ax.set_title(workload.replace('_', ' ').title(), fontsize=12, fontweight='bold')
        ax.grid(True, alpha=0.3, linestyle='--')
        ax.legend(fontsize=10)
        ax.set_xticks(workload_data['dataset_size_mb'].unique())
    
    plt.tight_layout()
    output_path = Path(output_dir) / "all_workloads_comparison.png"
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"✓ Saved: {output_path}")
    plt.close()


def print_summary(df):
    """Print summary statistics from the results."""
    print("\n" + "="*60)
    print("Dataset Scaling Experiment Summary")
    print("="*60)
    
    print(f"\nTotal experiments: {len(df)}")
    print(f"Dataset sizes: {sorted(df['dataset_size_mb'].unique())} MB")
    print(f"Workloads: {', '.join(sorted(df['experiment'].unique()))}")
    print(f"Execution modes: {', '.join(sorted(df['mode'].unique()))}")
    
    print("\nRuntime Range:")
    print(f"  Min: {df['runtime_ms'].min():.1f}ms")
    print(f"  Max: {df['runtime_ms'].max():.1f}ms")
    
    print("\nRuntime by Workload and Mode:")
    summary = df.groupby(['experiment', 'mode'])['runtime_ms'].agg(['min', 'mean', 'max'])
    print(summary)
    
    print("\n" + "="*60 + "\n")


def main():
    """Main entry point."""
    results_file = Path("results/rpi-results/dataset_scaling.jsonl")
    output_dir = Path("viz/output")
    
    # Create output directory if it doesn't exist
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print("Reading experiment results...")
    df = load_results(results_file)
    
    df["dataset_size_mb"] = df["working_set_bytes"] // (1024 * 1024)

    print_summary(df)
    
    print("Generating plots...")
    
    # Plot each workload
    for workload in sorted(df['experiment'].unique()):
        plot_workload_scaling(df, workload, output_dir)
    
    # Generate comparison plot
    plot_comparison(df, output_dir)
    
    print("\n✓ Visualization complete!")


if __name__ == "__main__":
    main()
