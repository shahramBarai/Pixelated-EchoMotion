## 1. Abstract

Pixelated EchoMotion is an interactive art project that explores the intersection of human motion, digital representation, and real-time interaction. Inspired by Rafael Lozano-Hemmer's "Body Movies," the project invites participants to dynamically engage with pixelated silhouettes of themselves and others, captured across time and locations. Through video processing, pixel art, and particle-based interaction, the project fosters creativity and connection by transforming human actions into abstract, interactive forms.

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
   - 1.1. Process the video frame by frame.
   - 1.2. If effect is applied to the silhouette in the frame -> particle system is created -> video frame is ignored until the particle system is finished.
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

## 4. Implementation and Progress

The implementation of Pixelated EchoMotion is still in progress, with the core components being developed and tested in isolation. Due to technical challenges and time constraints, a full prototype has not been completed. However, the following components have been developed and tested:

1. **Video Capture and Processing**: The project can capture short video clips and process them to extract human silhouettes. The OpenCV library is used to handle video input and frame processing, converting frames to grayscale, applying threshold filters, and extracting silhouettes.

2. **Silhouette Extraction and Pixelization**: The project can extract silhouettes from processed frames and convert them into pixel animations. The extracted silhouettes are pixelized and converted into particle systems for interaction. The particle systems can be manipulated to create effects like grabbing, repelling, and pixel explosions.

3. **Interaction Detection**: The project can detect interactions between silhouettes based on the distance between two points. When two silhouettes come close enough, an interaction is triggered, and the particle system responds accordingly.

4. **Output Rendering**: The project can render the pixelated silhouettes and particle systems on the screen, creating a visual representation of human actions and
   interactions.

The project has been tested in a small scale, with recorded videos and real-time interactions demonstrating the core concepts of Pixelated EchoMotion. The technical implementation has been challenging due to the complexity of real-time video processing and interaction systems. However, the project has made progress in developing the core components and laying the groundwork for a full prototype.

## 5. Outcomes

You can find the images and videos of the project in the `report/media` folder.

## 6. Personal Learning Goals

Working on this project has been a valuable learning experience, allowing me to explore new technologies, experiment with creative ideas, and develop technical skills. Some of the key learning goals achieved through this project include:

- **Rust Programming**: Gaining proficiency in Rust programming language, understanding its syntax, memory management, and concurrency features.
- **Real-time Processing**: Exploring real-time video processing and interaction systems, learning how to optimize performance and memory usage.
- **Creative Coding**: Experimenting with creative coding techniques, such as pixel art, particle systems, and interactive installations.
- **Art and Technology**: Exploring the intersection of art and technology, blending digital representation with human interaction in a creative context.

Overall, the project has been a rewarding experience, pushing me to explore new ideas, learn new technologies, and develop my skills as a creative coder and technologist.

## 7. Future Directions

This project has the potential to evolve into a dynamic and engaging interactive installation that connects human actions across time and space. Some of the future directions for Pixelated EchoMotion include:

- **Full Prototype Development**: Completing the technical implementation of the project to create a full prototype that can be exhibited in public spaces.
- **Interactive Exhibitions**: Showcasing Pixelated EchoMotion in exhibitions or events where participants can engage with the installation in a shared environment.
- **Collaborative Interactions**: Exploring collaborative interactions between participants, allowing them to create and interact with pixelated silhouettes together.
- **Dynamic Effects**: Adding more dynamic effects to the particle systems, such as color changes, size variations, and movement patterns.
- **Performance Optimization**: Optimizing the performance and memory usage of the project to ensure smooth operation and minimal delay in real-time interactions with webcam.

## 8. Conclusion

I learned a lot from this project, both technically and artistically. I explored new technologies, experimented with creative ideas, and developed my skills as a creative coder and technologist. While the project is still in progress, I am excited about the potential of Pixelated EchoMotion to create engaging and interactive experiences that connect human actions in unique and creative ways.

Even though the project is still in the development phase, I am proud of the progress made so far and look forward to continuing to work on it in the future if possible. I believe that this project has the potential to be a dynamic and engaging interactive installation that fosters creativity, connection, and playfulness through abstract digital art and real-time interaction systems.

## Links

1. Rafael Lozano-Hemmer's "Body Movies": https://www.lozano-hemmer.com/body_movies.php
2. Rust Programming Language: https://www.rust-lang.org/
3. Tokio library: https://tokio.rs/
4. OpenCV Rust Library: https://github.com/twistedfall/opencv-rust?tab=readme-ov-file
