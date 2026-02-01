import json
import base64
import struct

def generate_jet_gltf(output_path):
    # Vertices (x, y, z)
    # Nose points -Z
    vertices = [
        # Body
        (0.0, 0.0, -2.0),   # 0: Nose
        (0.0, 0.5, -0.5),   # 1: Cockpit top
        (0.3, 0.0, 1.0),    # 2: Rear Right
        (-0.3, 0.0, 1.0),   # 3: Rear Left
        (0.0, -0.2, 0.0),   # 4: Belly
        
        # Wings
        (2.0, 0.0, 0.5),    # 5: Right Wing Tip
        (-2.0, 0.0, 0.5),   # 6: Left Wing Tip
        (0.5, 0.0, -0.5),   # 7: Right Wing Root
        (-0.5, 0.0, -0.5),  # 8: Left Wing Root
        
        # Tail
        (0.0, 0.8, 1.0),    # 9: Tail Top
        (0.0, 0.0, 0.5),    # 10: Tail Base
    ]

    # Triangles (indices)
    indices = [
        # Fuselage
        (0, 1, 7), (0, 7, 2), (0, 2, 4), (0, 4, 3), (0, 3, 8), (0, 8, 1),
        (1, 2, 7), (1, 3, 2), (1, 8, 3), 
        (4, 2, 3), # Bottom rear
        
        # Wings
        (7, 5, 2), (8, 3, 6),
        
        # Tail
        (10, 9, 2), (10, 3, 9)
    ]

    # Flatten data
    v_data = b""
    for v in vertices:
        v_data += struct.pack("fff", *v)
    
    i_data = b""
    for tri in indices:
        i_data += struct.pack("HHH", *tri)

    # Alignment
    while len(v_data) % 4 != 0: v_data += b"\x00"
    
    v_offset = 0
    v_len = len(v_data)
    i_offset = v_len
    i_len = len(i_data)
    
    full_data = v_data + i_data
    encoded_data = base64.b64encode(full_data).decode("ascii")
    uri = f"data:application/octet-stream;base64,{encoded_data}"

    gltf = {
        "asset": {"version": "2.0", "generator": "Gemini-Procedural"},
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{
            "primitives": [{
                "attributes": {"POSITION": 1},
                "indices": 0,
                "material": 0
            }]
        }],
        "materials": [{
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.7, 0.7, 0.8, 1.0],
                "metallicFactor": 0.8,
                "roughnessFactor": 0.2
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
                "max": [max(v[0] for v in vertices), max(v[1] for v in vertices), max(v[2] for v in vertices)],
                "min": [min(v[0] for v in vertices), min(v[1] for v in vertices), min(v[2] for v in vertices)]
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

    with open(output_path, "w") as f:
        json.dump(gltf, f, indent=2)

if __name__ == "__main__":
    generate_jet_gltf("C:/Users/Box/plane_game/assets/models/fighter_jet.gltf")
