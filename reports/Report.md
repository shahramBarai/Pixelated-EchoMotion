## 1. Abstract

Pixelated EchoMotion is an interactive art project that explores the intersection of human motion, digital representation, and real-time interaction. Inspired by Rafael Lozano-Hemmer's "Body Movies," the project invites participants to dynamically engage with pixelated silhouettes of themselves and others, captured across time and locations. Through video processing, pixel art, and particle-based interaction, the project fosters creativity and connection by transforming human actions into abstract, interactive forms. This report outlines the project's motivation, technical overview, implementation progress, personal learning goals, and future directions.

## 2. Introduction

Pixelated EchoMotion blends computational art with interactive installations, drawing inspiration from Rafael Lozano-Hemmer's "Body Movies" [[1]](https://www.lozano-hemmer.com/body_movies.php). This project reimagines human silhouettes as pixelated animations that participants can dynamically engage with in real time or across different moments in time. By transforming recorded actions into pixelated, interactive forms, the project encourages participants to connect and play with digital representations of themselves and others in a shared space with total freedom of movement and interaction and with small randomnes in the pixelated silhouettes. The project aims to foster creativity, connection, and playfulness through abstract digital art and real-time interaction systems.

The motivation behind project stems from both artistic and technical interests. The project seeks to explore how abstract digital representation of human actions can facilitate new forms of interaction, creativity and engagement, by combining video processing, pixel art, and particle-based interaction. The project also aims to experiment with new technologies, such as Rust, OpenCV, and Tokio, to optimize performance and memory usage in real-time video processing and interaction systems. One of the key goals of the project is to make the interaction as smooth as possible, with minimal delay and high responsiveness, by leveraging parallel processing and efficient algorithms.

The core workflow of Pixelated EchoMotion involves capturing short video clips of human actions, pixelizing these clips to extract silhouettes and convert them into pixel animations, and enabling real-time interaction between participants' silhouettes and prerecorded pixel animations. Ideally, Pixelated EchoMotion would be displayed in exhibitions or public spaces where participants can engage with the installation in a shared environment, where they can record their actions (_zone 1_) and interact with the pixelated silhouettes of others (_zone 2_). The project envisions a two-zone setup: one for recording and pixelization and another for real-time interaction with the pixelated animations. But during the lack of time and resources, the project is still in the conceptual and technical development phase. But the project was tested and recorded in a small scale.

## 3. Techniacal Overview

The technical implementation of Pixelated EchoMotion involves using Rust as the programming language for its performance and memory safety, Tokio for parallel processing to handle real-time interactions and video manipulation efficiently, and OpenCV for image and video processing to extract silhouettes and pixelize animations.

#### Tools and Libraries

- Programming Language: **Rust** [[2]](https://www.rust-lang.org/) (for performance and memory safety).
- Parallel Processing: **Tokio** [[3]](https://tokio.rs/), to handle real-time interactions and video manipulation efficiently.
- Image and Video Processing: **OpenCV**[[4]](https://github.com/twistedfall/opencv-rust?tab=readme-ov-file), for silhouette extraction and pixelization.
- Techniques:
  - Prerecorded and Real-time Processing: To capture, pixelize, and dynamically process human actions.
  - Particle System Design: For effects like grabbing, repelling, and pixel explosions.

As mentioned, one of the main reasons for using this specific stack was to learn and understand the Rust programming language and its libraries. Additionally:

- **Rust** was chosen for its performance, memory safety, and concurrency capabilities. It ensures the application remains efficient and reliable, even when processing multiple video streams in real time.
- **Tokio** enables efficient management of concurrent tasks, such as video capture, pixelization, and particle-based interactions. Its asynchronous nature ensures smooth operation without performance bottlenecks.
- **OpenCV** provides robust tools for silhouette extraction, image filtering, and pixel manipulation. Its Rust bindings allow seamless integration with the broader Rust-based architecture of the project.

#### Core Workflow

The core workflow of project can be broken down into four main steps:

1. Video Capture: Short clips of human actions (pre-recorded) and real-time interactions (live).
   - 1.1. Processing each frame of the videos to detect human silhouettes and extract them.
   - 1.2. If the interaction is detected and effect is applied to the silhouette in the frame -> particle system is created -> video frame is ignored until the particle system is finished.
2. Frame Processing: The recorded videos are processed to detect human silhouettes.
   - 2.1. Converting to grayscale to simplify silhouette extraction.
   - 2.2. Threshold filtering to remove background noise.
   - 2.3. Converting frame to black and white to highlight silhouettes.
3. Detecting interactions: Using frame processing results to detect interactions between silhouettes.
   - 3.1. Finding contours to identify human shapes.
   - 3.2. Finding two closest points of silhouettes to detect interactions.
   - 3.3. Calculating the distance between two points to determine interaction.
4. Silhouette Extraction: Extracting silhouettes from processed frames and converting them into pixel animations if 3.3 is true.
   - 4.1. Extracting silhouettes from processed frames.
   - 4.2. Pixelizing silhouettes and creating particle systems for interaction.
   - 4.3. Applying one of the effects to the particle system.
   - 4.4. Updating the particle system each frame.
   - 4.5. Adding pixelated silhouettes to output frame.
5. Drowing the output: Drawing the pixelated silhouettes and particle systems on the screen.

<!-- ## 4. Implementation and Progress

Current Progress

- Learning Phase: Time has primarily been spent understanding Rust, OpenCV, and the intricacies of real-time video and particle-based systems.
- Technical Setup: Preliminary work on pixelization and interaction algorithms, though full-scale experiments are still pending.
- Conceptual Planning: Detailed descriptions and visual designs of the two-zone setup for exhibitions have been developed.

Challenges

- Developing real-time interaction systems within the constraints of performance and memory.
- Limited time for experimentation due to the steep learning curve associated with Rust and new libraries.

Planned Enhancements

- Detailed descriptions of the interaction process and ideal setups for exhibitions.
- Illustrative diagrams showing how zones are designed to foster engagement.

## 5. Outcomes

While technical hurdles have prevented the creation of a full prototype, the conceptual work lays the groundwork for:

- A scalable system capable of real-time silhouette interaction.
- A design philosophy centered on creativity, engagement, and the interplay between recorded and live actions.
- A vision for an exhibition where human silhouettes transcend time and physical boundaries.

## 6. Personal Learning Goals

1. Mastering Video Manipulation and Pixel Art:
   - Gained a deeper understanding of silhouette extraction and pixel processing.
2. Experimenting with Dynamic Interaction:
   - Explored the basics of collision detection and particle systems for interactive pixel animations.
3. Exploring Computational Art:
   - Conceptually expanded the boundaries of art by linking human actions across time and space.

## 7. Future Directions

Ideal Setup for Exhibition

- Zone 1:
  - Participants record short videos (10-30 seconds) of their actions.
  - These videos are processed into pixel animations and stored in a shared repository.
- Zone 2:
  - Participants interact with the stored animations through a camera and projector setup.
  - Past silhouettes from previous exhibitions encourage interactions that transcend time and geography.

Technical Improvements

- Real-time optimization of the silhouette extraction and particle systems.
- Deployment of a scalable setup capable of running in exhibition spaces with minimal delay.

## 8. Conclusion

Pixelated EchoMotion is a creative exploration of human interaction through the lens of digital representation. Despite technical challenges, the project has opened new avenues for blending art, technology, and interactivity. The vision of connecting human actions across time and space continues to guide its evolution toward a dynamic and engaging installation. -->

## Links

1. Rafael Lozano-Hemmer's "Body Movies": https://www.lozano-hemmer.com/body_movies.php
2. Rust Programming Language: https://www.rust-lang.org/
3. Tokio library: https://tokio.rs/
4. OpenCV Rust Library: https://github.com/twistedfall/opencv-rust?tab=readme-ov-file
