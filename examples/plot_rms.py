from pathlib import Path

import matplotlib.animation as animation
import matplotlib.colorbar as colorbar
import matplotlib.pyplot as plt
import numpy as np
import polars as pl
from matplotlib.colors import Normalize


def plot_focus():
    df = pl.read_csv(Path(__file__).parent.parent / "rms_focus.csv")
    t = float(df.columns[3].replace("rms[Pa]@", "").replace("[ns]", "")) / 1000_000
    rms = df.get_columns()[3]

    x = np.unique(df["x[mm]"])
    y = np.unique(df["y[mm]"])
    rms = rms.to_numpy().reshape([len(y), len(x)])
    aspect = (len(x), len(y), len(x))
    x, y = np.meshgrid(x, y)

    fig = plt.figure()
    spec = fig.add_gridspec(ncols=2, nrows=1, width_ratios=[10, 1])
    ax = fig.add_subplot(spec[0], projection="3d")
    cax = fig.add_subplot(spec[1])
    ax.plot_surface(
        x,
        y,
        rms,
        shade=False,
        cmap="jet",
        norm=Normalize(vmin=0, vmax=rms.max()),
    )
    ax.set_box_aspect(aspect)
    ax.set_title(f"t={t} [ms]")
    colorbar.ColorbarBase(cax, cmap="jet", norm=Normalize(vmin=0, vmax=rms.max()))
    plt.show()


def plot_stm():
    df = pl.read_csv(Path(__file__).parent.parent / "rms_stm.csv")
    times = [float(c.replace("rms[Pa]@", "").replace("[ns]", "")) / 1000_000 for c in df.columns[3:]]
    rms = df.get_columns()[3:]
    times = times[70:]
    rms = rms[70:]

    fig = plt.figure()
    spec = fig.add_gridspec(ncols=2, nrows=1, width_ratios=[10, 1])
    ax = fig.add_subplot(spec[0], projection="3d")
    cax = fig.add_subplot(spec[1])
    colorbar.ColorbarBase(cax, cmap="jet", norm=Normalize(vmin=-0, vmax=5e3))

    x = np.unique(df["x[mm]"])
    y = np.unique(df["y[mm]"])
    rms_shape = [len(y), len(x)]
    aspect = (len(x), len(y), len(x))
    x, y = np.meshgrid(x, y)

    def f(i):
        ax.cla()
        z = rms[i].to_numpy().reshape(rms_shape)
        plot = ax.plot_surface(x, y, z, shade=False, cmap="jet", norm=Normalize(vmin=-0, vmax=5e3))
        ax.set_zlim(0e3, 5e3)
        ax.set_box_aspect(aspect)
        ax.set_title(f"t={times[i]:.3f} [ms]")
        return plot

    _ = animation.FuncAnimation(fig, f, frames=len(rms), interval=1, repeat=False, blit=False)
    plt.show()


if __name__ == "__main__":
    plot_focus()
    plot_stm()
