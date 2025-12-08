// Mystical Particle System using p5.js
let particles = [];
let particleCount = 50;

function setup() {
    let canvas = createCanvas(windowWidth, windowHeight);
    canvas.parent('particle-container');
    canvas.style('position', 'fixed');
    canvas.style('top', '0');
    canvas.style('left', '0');
    canvas.style('pointer-events', 'none');
    canvas.style('z-index', '1');
    
    // Initialize particles
    for (let i = 0; i < particleCount; i++) {
        particles.push(new Particle());
    }
}

function draw() {
    clear();
    
    // Update and display particles
    for (let particle of particles) {
        particle.update();
        particle.display();
    }
}

function windowResized() {
    resizeCanvas(windowWidth, windowHeight);
}

class Particle {
    constructor() {
        this.x = random(width);
        this.y = random(height);
        this.vx = random(-0.5, 0.5);
        this.vy = random(-0.5, 0.5);
        this.size = random(2, 6);
        this.opacity = random(0.1, 0.3);
        this.color = random(['#d4a574', '#6b5b95', '#4a7c59']);
        this.life = random(100, 200);
        this.maxLife = this.life;
    }
    
    update() {
        this.x += this.vx;
        this.y += this.vy;
        this.life--;
        
        // Fade out as life decreases
        this.opacity = map(this.life, 0, this.maxLife, 0, 0.3);
        
        // Respawn particle when it dies
        if (this.life <= 0) {
            this.x = random(width);
            this.y = random(height);
            this.life = this.maxLife;
            this.opacity = random(0.1, 0.3);
        }
        
        // Wrap around edges
        if (this.x < 0) this.x = width;
        if (this.x > width) this.x = 0;
        if (this.y < 0) this.y = height;
        if (this.y > height) this.y = 0;
    }
    
    display() {
        push();
        translate(this.x, this.y);
        
        // Create glowing effect
        drawingContext.shadowColor = this.color;
        drawingContext.shadowBlur = 10;
        
        fill(red(this.color), green(this.color), blue(this.color), this.opacity * 255);
        noStroke();
        ellipse(0, 0, this.size);
        
        pop();
    }
}