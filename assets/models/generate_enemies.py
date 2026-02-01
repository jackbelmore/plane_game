import json
import base64
import struct
import math
import os

def write_gltf(filename, vertices, indices, color, metallic=0.1, roughness=0.6, name="Enemy"):
    # Flatten data
    v_data = b""
    for v in vertices:
        v_data += struct.pack("fff", *v)
    
    i_data = b""
    for tri in indices:
        i_data += struct.pack("HHH", *tri)

    # Alignment padding
    while len(v_data) % 4 != 0: v_data += b"\x00"
    while len(i_data) % 4 != 0: i_data += b"\x00"
    
    v_offset = 0
    v_len = len(v_data)
    i_offset = v_len
    i_len = len(i_data)
    
    full_data = v_data + i_data
    encoded_data = base64.b64encode(full_data).decode("ascii")
    uri = f"data:application/octet-stream;base64,{encoded_data}"

    # Bounding box
    xs = [v[0] for v in vertices]
    ys = [v[1] for v in vertices]
    zs = [v[2] for v in vertices]

    gltf = {
        "asset": {"version": "2.0", "generator": "Gemini-Procedural-Enemy"},
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": name}],
        "meshes": [{
            "name": name,
            "primitives": [{
                "attributes": {"POSITION": 1},
                "indices": 0,
                "material": 0
            }]
        }],
        "materials": [{
            "pbrMetallicRoughness": {
                "baseColorFactor": color,
                "metallicFactor": metallic,
                "roughnessFactor": roughness
            }
        }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5123, # UNSIGNED_SHORT
                "count": len(indices) * 3,
                "type": "SCALAR"
            },
            {
                "bufferView": 1,
                "componentType": 5126, # FLOAT
                "count": len(vertices),
                "type": "VEC3",
                "max": [max(xs), max(ys), max(zs)],
                "min": [min(xs), min(ys), min(zs)]
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": i_offset,
                "byteLength": i_len,
                "target": 34963 # ELEMENT_ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": v_offset,
                "byteLength": v_len,
                "target": 34962 # ARRAY_BUFFER
            }
        ],
        "buffers": [{
            "byteLength": len(full_data),
            "uri": uri
        }]
    }

    with open(filename, "w") as f:
        json.dump(gltf, f, indent=2)
    print(f"Generated {filename}")

def generate_red_aggressor():
    # Delta wing, aggressive, pointy
    vertices = [
        (0.0, 0.0, -2.5),   # 0: Long Nose
        (0.0, 0.4, -0.5),   # 1: Cockpit
        (0.4, 0.0, 1.0),    # 2: Rear R
        (-0.4, 0.0, 1.0),   # 3: Rear L
        (0.0, -0.3, 0.5),   # 4: Belly
        # Wings (Sharp Delta)
        (2.2, 0.0, 1.0),    # 5: Wing R Tip (Back)
        (-2.2, 0.0, 1.0),   # 6: Wing L Tip (Back)
        (0.5, 0.0, -1.0),   # 7: Wing Root R
        (-0.5, 0.0, -1.0),  # 8: Wing Root L
        # Tail (Dual vertical stabilizers)
        (0.3, 0.8, 1.0),    # 9: Tail R Top
        (-0.3, 0.8, 1.0),   # 10: Tail L Top
    ]
    
    indices = [
        # Body
        (0,1,7), (0,7,2), (0,2,4), (0,4,3), (0,3,8), (0,8,1),
        (1,2,7), (1,3,2), (1,8,3), (4,2,3),
        # Wings
        (7,5,2), (8,3,6),
        # Tails
        (2,9,3), (3,10,2) # Rough approx for dual tail
    ]
    
    # Matte Red
    write_gltf("assets/models/enemy_red.gltf", vertices, indices, [0.8, 0.1, 0.1, 1.0], 0.2, 0.8, "RedAggressor")

def generate_green_defender():
    # Boxy, sturdy, straight wings
    vertices = [
        (0.0, 0.0, -1.8),   # 0: Blunt Nose
        (0.0, 0.6, -0.2),   # 1: High Cockpit
        (0.5, 0.0, 1.2),    # 2: Rear R
        (-0.5, 0.0, 1.2),   # 3: Rear L
        (0.0, -0.4, 0.5),   # 4: Deep Belly
        # Wings (Broad Rectangular)
        (2.5, 0.0, 0.0),    # 5: Wing R Tip Front
        (-2.5, 0.0, 0.0),   # 6: Wing L Tip Front
        (2.5, 0.0, 0.8),    # 7: Wing R Tip Back
        (-2.5, 0.0, 0.8),   # 8: Wing L Tip Back
        (0.5, 0.0, 0.0),    # 9: Root R Front
        (-0.5, 0.0, 0.0),   # 10: Root L Front
        # Tail
        (0.0, 1.0, 1.2),    # 11: Tail Top
    ]
    
    indices = [
        # Body
        (0,1,2), (0,2,4), (0,4,3), (0,3,1), (4,2,3), (1,3,2),
        # Wings
        (9,5,7), (9,7,2), # Right
        (10,8,6), (10,3,8), # Left
        # Tail
        (1,11,2), (1,3,11)
    ]
    
    # Olive Green
    write_gltf("assets/models/enemy_green.gltf", vertices, indices, [0.3, 0.4, 0.2, 1.0], 0.3, 0.9, "GreenDefender")

def generate_black_stealth():
    # Sleek, flattened, forward swept looks
    vertices = [
        (0.0, 0.0, -3.0),   # 0: Very Long Nose
        (0.0, 0.3, -0.5),   # 1: Low Cockpit
        (0.6, 0.0, 0.8),    # 2: Rear R
        (-0.6, 0.0, 0.8),   # 3: Rear L
        (0.0, -0.1, 0.0),   # 4: Flat Belly
        # Wings (Swept back aggressively)
        (1.8, -0.1, 0.8),   # 5: Wing R Tip
        (-1.8, -0.1, 0.8),  # 6: Wing L Tip
        (0.4, 0.0, -1.0),   # 7: Wing Root R
        (-0.4, 0.0, -1.0),  # 8: Wing Root L
    ]
    
    indices = [
        # Body (Diamond shape cross section)
        (0,1,7), (0,7,2), (0,2,4), (0,4,3), (0,3,8), (0,8,1),
        (1,2,7), (1,3,2), (1,8,3), (4,2,3),
        # Wings
        (7,5,2), (8,3,6)
    ]
    
    # Matte Black
    write_gltf("assets/models/enemy_black.gltf", vertices, indices, [0.1, 0.1, 0.15, 1.0], 0.1, 0.4, "BlackStealth")

if __name__ == "__main__":
    if not os.path.exists("assets/models"):
        os.makedirs("assets/models")
    generate_red_aggressor()
    generate_green_defender()
    generate_black_stealth()
