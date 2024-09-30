from pathlib import Path

import matplotlib.animation as animation
import matplotlib.colorbar as colorbar
import matplotlib.pyplot as plt
import numpy as np
import polars as pl
from matplotlib.colors import Normalize
from scipy.interpolate import griddata


def plot_focus():
    df = pl.read_csv(Path(__file__).parent.parent / "sound_field_around_focus.csv")
    times = [float(c.replace("p[Pa]@", "")) * 1000 for c in df.columns[3:]]
    p = df.get_columns()[3:]
    times = times[440:]
    p = p[440:]

    fig = plt.figure()
    spec = fig.add_gridspec(ncols=2, nrows=1, width_ratios=[10, 1])
    ax = fig.add_subplot(spec[0], projection="3d")
    cax = fig.add_subplot(spec[1])
    colorbar.ColorbarBase(cax, cmap="jet", norm=Normalize(vmin=-10e3, vmax=10e3))

    x, y = np.meshgrid(np.unique(df["x[mm]"]), np.unique(df["y[mm]"]))

    def f(i):
        ax.cla()
        z = griddata((df["x[mm]"], df["y[mm]"]), p[i], (x, y))
        plot = ax.plot_surface(
            x, y, z, shade=False, cmap="jet", norm=Normalize(vmin=-10e3, vmax=10e3)
        )
        ax.set_zlim(-10e3, 10e3)
        ax.set_title(f"t={times[i]:.3f} [ms]")
        return plot

    _ = animation.FuncAnimation(
        fig, f, frames=len(p), interval=1, repeat=False, blit=False
    )
    plt.show()

    # plot RMS
    fig = plt.figure()
    spec = fig.add_gridspec(ncols=2, nrows=1, width_ratios=[10, 1])
    ax = fig.add_subplot(spec[0], projection="3d")
    cax = fig.add_subplot(spec[1])
    rms = (
        df.select(pl.exclude(r"^.\[mm\]$"))
        .select(pl.all().pow(2))
        .mean_horizontal()
        .sqrt()
    )
    ax.plot_surface(
        x,
        y,
        griddata((df["x[mm]"], df["y[mm]"]), rms, (x, y)),
        shade=False,
        cmap="jet",
        norm=Normalize(vmin=0, vmax=rms.max()),
    )
    colorbar.ColorbarBase(cax, cmap="jet", norm=Normalize(vmin=0, vmax=rms.max()))
    plt.show()


def plot_stm():
    df = pl.read_csv(Path(__file__).parent.parent / "sound_field_stm.csv")
    print(df)
    times = [float(c.replace("p[Pa]@", "")) * 1000 for c in df.columns[3:]]
    p = df.get_columns()[3:]
    times = times[700:]
    p = p[700:]

    fig = plt.figure()
    spec = fig.add_gridspec(ncols=2, nrows=1, width_ratios=[10, 1])
    ax = fig.add_subplot(spec[0], projection="3d")
    cax = fig.add_subplot(spec[1])
    colorbar.ColorbarBase(cax, cmap="jet", norm=Normalize(vmin=-10e3, vmax=10e3))

    x, y = np.meshgrid(np.unique(df["x[mm]"]), np.unique(df["y[mm]"]))

    def f(i):
        ax.cla()
        z = griddata((df["x[mm]"], df["y[mm]"]), p[i], (x, y))
        plot = ax.plot_surface(
            x, y, z, shade=False, cmap="jet", norm=Normalize(vmin=-10e3, vmax=10e3)
        )
        ax.set_zlim(-10e3, 10e3)
        ax.set_title(f"t={times[i]:.3f} [ms]")
        return plot

    _ = animation.FuncAnimation(
        fig, f, frames=len(p), interval=1, repeat=False, blit=False
    )
    plt.show()


if __name__ == "__main__":
    plot_focus()
    plot_stm()
