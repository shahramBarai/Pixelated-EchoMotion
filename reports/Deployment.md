# Deployment

This docoment is a deployment report for the project. It contains the deployment process, the tools used, and the deployment environment. Since I am using Ubuntu as my primary operating system, this document mainly focuses on the deployment process on Linux, but I will provide some links for Windows and Mac users as well if you are interested in deploying the project on those operating systems.

## Deployment Process

### Step 1: Install Rust

Let's start off by installing cargo. Cargo is the Rust package manager and it is used to build and run Rust projects. You can install cargo by running the following command (for other operating systems, you can check the installation guide [here](https://www.rust-lang.org/tools/install)):

```bash
curl https://sh.rustup.rs -sSf | sh
```

It will download and install the latest version of Rust and Cargo. After installing cargo, you can check the version of cargo by running the following command:

```bash
cargo --version
```

### Step 2: Install OpenCV

Next, we need to install OpenCV. OpenCV is an open-source computer vision and machine learning software library. You can install OpenCV by running the following command (for other operating systems, you can check the installation guide [here](https://github.com/twistedfall/opencv-rust/blob/master/INSTALL.md)):

```bash
sudo apt-get install libopencv-dev clang libclang-dev
```

### Step 3: Install SDL2

Next, we need to install SDL2. SDL2 is a cross-platform development library designed to provide low-level access to audio, keyboard, mouse, joystick, and graphics hardware via OpenGL and Direct3D. You can install SDL2 by running the following command.

```bash
sudo apt install libasound2-dev libudev-dev
```

### Step 4: Install dependencies

Now, we need to install the dependencies for the project. You can install the dependencies by running the following command:

```bash
cd computational_art_and_design
cargo build
```

### Step 5: Run the project

Finally, you can run the project by running the following command:

```bash
cargo run -- file data/punch.mp4 data/video_sources
```

This will run the project and runs the punch video as the main video source! If you want to use your `webcam` as the main video source, you can run the following command:

```bash
cargo run -- webcam <webcam_id> data/video_sources
```

**Note:** You need to have a white background to use the webcam as the main video source and ajust the webcam brightness and contrast to get better results (the main.rs file line 31 and 32).

Command line arguments:

```bash
cargo run -- [webcam <webcam_index> | file <video_path_1>] <folder_for_video_sources> [print_info | print_time_logs]
```

- `webcam <webcam_index>`: Use the webcam as the main video source. You can specify the webcam index to use a specific webcam. (optional if main video source file)
- `file <video_path_1>`: Use the video file as the main video source. You can specify the video path to use a specific video file. (optional if main video source webcam)
- `<folder_for_video_sources>`: The folder where the video sources will be saved. (**required**)
- `print_info`: Print the information about the video sources. (optional)
- `print_time_logs`: Print the time logs for the video sources (optional).

## Settings

You can change the settings in the `main.rs` file to adjust the pixel size, pixel spacing, window name, window width, window height, video resolution width, video resolution height, objects interference distance, webcam contrast, and webcam brightness. You can change these settings according to your requirements. By default:

- `PIXEL_SIZE` is set to `10`
- `PIXEL_SPACING` is set to `0`
- `WINDOW_NAME` is set to `"Window"`
- `WINDOW_WIDTH` is set to `960`
- `WINDOW_HEIGHT` is set to `540`
- `VIDEO_RESOLUTION_WIDTH` is set to `1920`
- `VIDEO_RESOLUTION_HEIGHT` is set to `1080`
- `OBJECTS_INTERFERENCE_DISTANCE` is set to `10`
- `WEBCAM_CONTRAST` is set to `1.0`
- `WEBCAM_BRIGHTNESS` is set to `90.0`

You can change these settings according to experiment with the project.
