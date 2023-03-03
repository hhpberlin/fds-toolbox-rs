
    "TITLE" => title = Some(parse!(next()? => full_line_string)?),
    "VERSION" | "FDSVERSION" => {
        fds_version = Some(parse!(next()? => full_line_string)?)
    }
    "ENDF" => end_file = Some(parse!(next()? => full_line_string)?),
    "INPF" => input_file = Some(parse!(next()? => full_line_string)?),
    "REVISION" => revision = Some(parse!(next()? => full_line_string)?),
    "CHID" => chid = Some(parse!(next()? => full_line_string)?),
    "SOLID_HT3D" => solid_ht3d = Some(parse!(next()? => i32)?),
    "CSVF" => {
        // TODO
        let name = parse!(next()? => full_line_string)?;
        let file = parse!(next()? => full_line_string)?;
        csv_files.insert(name, file);
    }
    "NMESHES" => num_meshes = Some(parse!(next()? => u32)?),
    "HRRPUVCUT" => hrrpuv_cutoff = Some(parse!(next()? => f32)?),
    "VIEWTIMES" => view_times = Some(parse!(next()? => f32 f32 i32)?),
    "ALBEDO" => albedo = Some(parse!(next()? => f32)),
    "IBLANK" => i_blank = Some(parse!(next()? => i32)),
    "GVEC" => g_vec = Some(parse!(next()? => vec3f)),
    "SURFDEF" => surfdef = Some(parse!(next()? => full_line_string)),
    "SURFACE" => {
        // TODO
        let name = parse!(next()? => full_line_string)?;
        let (tmpm, material_emissivity) = parse!(next()? => f32 f32)?;
        let (surface_type, texture_width, texture_height, rgb, transparency) =
            parse!(next()? => i32 f32 f32 vec3f f32)?;
        let texture = alt(("null".map(|_| None), full_line_string.map(Some)));
        let texture = parse!(next()? => texture)?;
        let surface = Surface {
            name,
            tmpm,
            material_emissivity,
            surface_type,
            texture_width,
            texture_height,
            rgb,
            transparency,
            texture,
        };
        surfaces.push(surface);
    }
    "MATERIAL" => {
        // Located::new("a").as_ref()
        // TODO
        let name = parse!(next()? => full_line_string)?;
        let rgb = parse!(next()? => vec3f)?;
        materials.push(Material { name, rgb });
    }
    "OUTLINE" => outlines = Some(repeat(next, |next, _| parse!(next()? => bounds3f))?),
    // TODO: This is called offset but fdsreader treats it as the default texture origin
    //       Check if it actually should be the default value or if it should be a global offset
    "TOFFSET" => default_texture_origin = Some(parse!(next()? => vec3f)?),
    "RAMP" => {
        ramps = Some(repeat(next, |next, _| {
            // TODO
            let (_, name) = parse!(next()? => "RAMP:" full_line_string)?;
            // TODO: next is &mut &mut here, remove double indirection
            let vals = repeat(next, |next, _| parse!(next()? => f32 f32))?;
            Ok((name, vals))
        })?)
    }
    "PROP" => {
        // TODO: This is probably wrong, based on a sample size of two
        let _ = parse!(next()? => "null")?;
        let _ = parse!(next()? => "1")?;
        let _ = parse!(next()? => "sensor")?;
        let _ = parse!(next()? => "0")?;
    }
    "DEVICE" => {
        let name = take_till0("%").map(|x: &str| x.trim().to_string());
        let unit = ("%", full_line_string).map(|(_, x)| x);
        let (name, unit) = parse!(next()? => name unit)?;

        // TODO: This is a bit ugly
        let close = ws_separated!("%", "null").recognize();

        // TODO: idk what this is
        let bounds = ws_separated!("#", bounds3f).map(|(_, x)| x);
        let bounds = ws_separated!(opt(bounds), close).map(|(x, _)| x);

        // TODO: what are a and b?
        let (position, orientation, a, b, bounds) =
            parse!(next()? => vec3f vec3f i32 i32 bounds)?;

        devices.insert(
            name.clone(),
            Device {
                name,
                unit,
                position,
                orientation,
                a,
                b,
                bounds,
                activations: Vec::new(),
            },
        );
    }
    line => {
        let Some(line_first) = line.split_whitespace().next() else {
            return Err(err(line, ErrorKind::InvalidSection));
        };

        match line_first {
            "GRID" => {
                let default_texture_origin = default_texture_origin
                    .ok_or(Error::MissingSection { name: "TOFFSET" })?;
                let mesh = mesh::parse_mesh(line, default_texture_origin, next)?;
                meshes.push(mesh);
            }
            "SMOKF3D" | "SMOKG3D" => {
                let tag = tag(line_first);
                let (_, num) = parse!(line => tag i32)?;

                let smoke_type = match line_first {
                    "SMOKF3D" => Smoke3DType::F,
                    "SMOKG3D" => Smoke3DType::G,
                    _ => unreachable!(),
                };

                let file_name = parse!(next()? => full_line_string)?;
                let quantity = parse!(next()? => full_line_string)?;
                let name = parse!(next()? => full_line_string)?;
                let unit = parse!(next()? => full_line_string)?;

                smoke3d.push(Smoke3D {
                    smoke_type,
                    num,
                    file_name,
                    quantity,
                    name,
                    unit,
                });
            }
            "SLCF" | "SLCC" => {
                // TODO: a lot, this is completely different from fdsreaders implementation
                let tag = tag(line_first);
                let (_, mesh_index, _, slice_type, _, bounds) =
                    parse!(line => tag i32 "#" string "&" bounds3i)?;

                let cell_centered = match line_first {
                    "SLCF" => false,
                    "SLCC" => true,
                    _ => unreachable!(),
                };

                let file_name = parse!(next()? => full_line_string)?;
                let quantity = parse!(next()? => full_line_string)?;
                let name = parse!(next()? => full_line_string)?;
                let unit = parse!(next()? => full_line_string)?;

                slices.push(Slice {
                    mesh_index,
                    slice_type,
                    bounds,
                    cell_centered,
                    file_name,
                    quantity,
                    name,
                    unit,
                });
            }
            "DEVICE_ACT" => {
                let (_, device) = parse!(line => "DEVICE_ACT" full_line_str)?;

                let Some(device) = devices.get_mut(device) else {
                    return Err(Error::InvalidKey {
                        parent: line,
                        key: device,
                    });
                };

                let (a, b, c) = parse!(line => i32 f32 i32)?;

                device.activations.push(DeviceActivation { a, b, c });
            }
            "OPEN_VENT" | "CLOSE_VENT" => {
                let tag = tag(line_first);
                let (_, mesh_index) = parse!(line => tag i32)?;
                let (_a, _b) = parse!(line => i32 f32)?;

                // TODO
            }
            "PL3D" => {
                let (_, _a, num) = parse!(line => "PL3D" f32 i32)?;

                let file_name = parse!(next()? => full_line_string)?;
                let quantity = parse!(next()? => full_line_string)?;
                let name = parse!(next()? => full_line_string)?;
                let unit = parse!(next()? => full_line_string)?;

                // TODO: lines missing

                pl3d.push(Plot3D {
                    num,
                    file_name,
                    quantity,
                    name,
                    unit,
                });
            }
            _ => return Err(err(line, ErrorKind::UnknownSection)),
        }
    }
}