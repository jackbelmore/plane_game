"""
Enhanced F-16 Fighter Jet Model Generator
Generates a more detailed procedural GLTF with proper normals and better geometry
"""
import json
import base64
import struct
import math

def cross(a, b):
    """Vector cross product"""
    return (
        a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0]
    )

def normalize(v):
    """Normalize vector"""
    mag = math.sqrt(v[0]**2 + v[1]**2 + v[2]**2)
    if mag < 0.001:
        return (0, 1, 0)
    return (v[0]/mag, v[1]/mag, v[2]/mag)

def calculate_normals(vertices, indices):
    """Calculate smooth vertex normals"""
    normals = [[0.0, 0.0, 0.0] for _ in vertices]
    
    # For each triangle, calculate face normal and add to vertices
    for tri in indices:
        i0, i1, i2 = tri
        v0, v1, v2 = vertices[i0], vertices[i1], vertices[i2]
        
        # Edge vectors
        e1 = (v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2])
        e2 = (v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2])
        
        # Face normal (cross product)
        n = cross(e1, e2)
        
        # Add to each vertex
        for idx in tri:
            normals[idx][0] += n[0]
            normals[idx][1] += n[1]
            normals[idx][2] += n[2]
    
    # Normalize all
    normals = [normalize(tuple(n)) for n in normals]
    return normals

def generate_enhanced_f16():
    """Generate a more detailed F-16 with proper normals"""
    
    # More detailed geometry
    vertices = [
        # Nose section (pointed)
        (0.0, 0.0, -3.5),       # 0: Very front tip
        (0.0, 0.15, -3.0),      # 1: Nose top
        (0.15, 0.0, -3.0),      # 2: Nose right
        (-0.15, 0.0, -3.0),     # 3: Nose left
        (0.0, -0.15, -3.0),     # 4: Nose bottom
        
        # Cockpit area
        (0.0, 0.6, -1.5),       # 5: Cockpit peak
        (0.3, 0.3, -1.0),       # 6: Cockpit right
        (-0.3, 0.3, -1.0),      # 7: Cockpit left
        (0.3, 0.0, -0.5),       # 8: Body right front
        (-0.3, 0.0, -0.5),      # 9: Body left front
        
        # Fuselage mid
        (0.4, 0.2, 0.5),        # 10: Body right mid top
        (-0.4, 0.2, 0.5),       # 11: Body left mid top
        (0.4, -0.2, 0.5),       # 12: Body right mid bottom
        (-0.4, -0.2, 0.5),      # 13: Body left mid bottom
        
        # Rear fuselage
        (0.3, 0.1, 1.5),        # 14: Rear right top
        (-0.3, 0.1, 1.5),       # 15: Rear left top
        (0.3, -0.1, 1.5),       # 16: Rear right bottom
        (-0.3, -0.1, 1.5),      # 17: Rear left bottom
        
        # Engine exhaust
        (0.15, 0.0, 2.0),       # 18: Exhaust right
        (-0.15, 0.0, 2.0),      # 19: Exhaust left
        
        # Main wings (delta shape)
        (2.5, 0.0, 1.0),        # 20: Right wing tip
        (-2.5, 0.0, 1.0),       # 21: Left wing tip
        (0.8, 0.0, -0.5),       # 22: Right wing root front
        (-0.8, 0.0, -0.5),      # 23: Left wing root front
        (0.6, 0.0, 1.2),        # 24: Right wing root rear
        (-0.6, 0.0, 1.2),       # 25: Left wing root rear
        
        # Vertical stabilizer
        (0.0, 0.8, 1.3),        # 26: Tail top
        (0.0, 0.1, 0.8),        # 27: Tail base front
        (0.0, 0.1, 1.8),        # 28: Tail base rear
        
        # Horizontal stabilizers
        (0.8, 0.0, 1.6),        # 29: Right stab tip
        (-0.8, 0.0, 1.6),       # 30: Left stab tip
        (0.3, 0.0, 1.3),        # 31: Right stab root
        (-0.3, 0.0, 1.3),       # 32: Left stab root
    ]
    
    # Triangles (double-sided for solid model)
    front_faces = [
        # Nose cone (front)
        (0,1,2), (0,2,4), (0,4,3), (0,3,1),
        
        # Nose to cockpit
        (1,5,6), (1,6,2), (2,6,8), (3,7,1), (1,7,5), (3,9,7),
        (4,2,8), (4,3,9),
        
        # Cockpit to mid fuselage  
        (5,10,6), (5,11,7), (6,10,8), (7,11,9),
        (8,10,12), (9,13,11), (8,12,9), (9,12,13),
        
        # Mid to rear fuselage
        (10,14,12), (11,13,15), (12,14,16), (13,17,15),
        (14,18,16), (15,17,19),
        
        # Main wings (top)
        (22,20,24), (23,25,21), (8,22,24), (9,25,23),
        (24,20,16), (25,17,21), (12,16,24), (13,25,17),
        
        # Vertical stabilizer (right side)
        (27,26,28), (14,27,15), (14,26,27), (15,27,26),
        
        # Horizontal stabilizers (top)
        (31,29,16), (32,17,30), (16,29,18), (17,19,30),
        
        # Close fuselage sections
        (10,11,14), (11,15,14), (14,15,16), (15,17,16),
    ]
    
    # Add back faces (reversed winding)
    back_faces = [(t[0], t[2], t[1]) for t in front_faces]
    
    # Combine for solid double-sided model
    indices = front_faces + back_faces
    
    # Calculate normals
    normals = calculate_normals(vertices, indices)
    
    # Pack vertex data (position + normal interleaved)
    vertex_data = b""
    for i, v in enumerate(vertices):
        vertex_data += struct.pack("fff", *v)  # Position
        vertex_data += struct.pack("fff", *normals[i])  # Normal
    
    # Pack indices
    index_data = b""
    for tri in indices:
        index_data += struct.pack("HHH", *tri)
    
    # Alignment
    while len(vertex_data) % 4 != 0:
        vertex_data += b"\x00"
    while len(index_data) % 4 != 0:
        index_data += b"\x00"
    
    v_offset = 0
    v_len = len(vertex_data)
    i_offset = v_len
    i_len = len(index_data)
    
    full_data = vertex_data + index_data
    encoded = base64.b64encode(full_data).decode("ascii")
    uri = f"data:application/octet-stream;base64,{encoded}"
    
    # Bounding box
    xs = [v[0] for v in vertices]
    ys = [v[1] for v in vertices]
    zs = [v[2] for v in vertices]
    
    gltf = {
        "asset": {"version": "2.0", "generator": "Enhanced-F16-Generator"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": "F16_Fighter"}],
        "meshes": [{
            "name": "F16_Mesh",
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1
                },
                "indices": 2,
                "material": 0
            }]
        }],
        "materials": [{
            "name": "F16_Material",
            "pbrMetallicRoughness": {
                "baseColorFactor": [0.7, 0.75, 0.8, 1.0],  # Light gray-blue
                "metallicFactor": 0.9,
                "roughnessFactor": 0.3
            }
        }],
        "accessors": [
            # Position accessor
            {
                "bufferView": 0,
                "componentType": 5126,  # FLOAT
                "count": len(vertices),
                "type": "VEC3",
                "max": [max(xs), max(ys), max(zs)],
                "min": [min(xs), min(ys), min(zs)]
            },
            # Normal accessor
            {
                "bufferView": 1,
                "componentType": 5126,  # FLOAT
                "count": len(vertices),
                "type": "VEC3"
            },
            # Indices accessor
            {
                "bufferView": 2,
                "componentType": 5123,  # UNSIGNED_SHORT
                "count": len(indices) * 3,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            # Position buffer view (stride = 24 bytes: 3 pos + 3 normal floats)
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": v_len,
                "byteStride": 24,
                "target": 34962  # ARRAY_BUFFER
            },
            # Normal buffer view (same buffer, offset by 12 bytes)
            {
                "buffer": 0,
                "byteOffset": 12,
                "byteLength": v_len,
                "byteStride": 24,
                "target": 34962
            },
            # Index buffer view
            {
                "buffer": 0,
                "byteOffset": i_offset,
                "byteLength": i_len,
                "target": 34963  # ELEMENT_ARRAY_BUFFER
            }
        ],
        "buffers": [{
            "byteLength": len(full_data),
            "uri": uri
        }]
    }
    
    output_path = "C:/Users/Box/plane_game/assets/models/fighter_jet_enhanced.gltf"
    with open(output_path, "w") as f:
        json.dump(gltf, f, indent=2)
    
    print(f"✓ Generated enhanced F-16 model: {output_path}")
    print(f"  Vertices: {len(vertices)}")
    print(f"  Triangles: {len(indices)}")
    print(f"  Features: Smooth normals, detailed geometry, metallic material")

if __name__ == "__main__":
    generate_enhanced_f16()
