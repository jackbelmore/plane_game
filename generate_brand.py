from PIL import Image, ImageDraw, ImageFont
import os

def generate_logo():
    # Colors
    BG_COLOR = (15, 20, 25)  # Stealth Dark Blue/Gray
    HUD_GREEN = (0, 255, 65)  # Classic CRT Green
    GLOW_GREEN = (0, 100, 25, 100)
    WHITE = (255, 255, 255)

    # Size
    size = 512
    img = Image.new('RGB', (size, size), color=BG_COLOR)
    draw = ImageDraw.Draw(img)

    # 1. Draw Targeting Reticle (The "Eye")
    center = size // 2
    radius = 180
    
    # Outer circle
    draw.ellipse([center - radius, center - radius, center + radius, center + radius], outline=HUD_GREEN, width=4)
    
    # Inner dashed circles/markings
    for i in range(0, 360, 45):
        # Draw ticks
        import math
        angle = math.radians(i)
        x1 = center + (radius - 10) * math.cos(angle)
        y1 = center + (radius - 10) * math.sin(angle)
        x2 = center + (radius + 10) * math.cos(angle)
        y2 = center + (radius + 10) * math.sin(angle)
        draw.line([x1, y1, x2, y2], fill=HUD_GREEN, width=3)

    # 2. Draw F-16 Silhouette (Simplified)
    # Nose
    nose = (center, center - 80)
    wing_l = (center - 100, center + 40)
    wing_r = (center + 100, center + 40)
    tail_l = (center - 40, center + 100)
    tail_r = (center + 40, center + 100)
    base = (center, center + 80)

    points = [nose, wing_r, (center + 20, center + 40), tail_r, base, tail_l, (center - 20, center + 40), wing_l, nose]
    draw.polygon(points, fill=WHITE)

    # 3. Text
    # Since we can't guarantee font files, we draw block letters manually
    def draw_block_text(d, text, x, y, size_h, color):
        # Very simple 5x5 grid font logic for "VIPER EYE"
        # For simplicity in this script, we'll use draw.text with a default font if possible, 
        # but let's try a robust approach
        try:
            # Try to find a system font
            font = ImageFont.truetype("arial.ttf", 60)
            d.text((x, y), text, font=font, fill=color)
        except:
            # Fallback to simple lines if font fails
            d.text((x, y), text, fill=color)

    # Draw Text Background Plate
    draw.rectangle([center - 150, center + 140, center + 150, center + 200], fill=(0, 0, 0))
    draw.rectangle([center - 150, center + 140, center + 150, center + 200], outline=HUD_GREEN, width=2)
    
    # Text labels
    try:
        font = ImageFont.load_default()
        # Draw "VIPER EYE" at the bottom
        # Centering text is tricky with default, so we offset
        draw.text((center - 45, center + 155), "VIPER EYE", fill=HUD_GREEN)
    except:
        pass

    # Ensure directories exist
    os.makedirs('assets/ui', exist_ok=True)

    # Save outputs
    img.save('assets/ui/logo.png')
    
    # Icon version
    icon = img.resize((256, 256), Image.Resampling.LANCZOS).convert('RGBA')
    icon.save('assets/ui/icon.png')
    
    print("Logo and Icon generated in assets/ui/")

if __name__ == "__main__":
    generate_logo()
