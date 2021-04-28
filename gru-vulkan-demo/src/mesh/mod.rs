use crate::*;

pub fn generate() -> (Vec<u16>, Vec<Vertex>)
{
	let indices = vec!
	[
		2, 6, 4, 4, 0, 2, //f
		7, 3, 1, 1, 5, 7, //b
		3, 2, 0, 0, 1, 3, //l
		6, 7, 5, 5, 4, 6, //r
		0, 4, 5, 5, 1, 0, //u
		3, 7, 6, 6, 2, 3  //d
	];
    let vertices = vec!
    [
        Vertex { position: (-1.0, -1.0, -1.0).into(), color: (0.0, 0.0, 0.0).into() }, // luf 0
        Vertex { position: (-1.0, -1.0, 1.0).into(), color: (0.0, 0.0, 1.0).into() }, // lub 1
        Vertex { position: (-1.0, 1.0, -1.0).into(), color: (0.0, 1.0, 0.0).into() }, // ldf 2
        Vertex { position: (-1.0, 1.0, 1.0).into(), color: (0.0, 1.0, 1.0).into() }, // ldb 3
        Vertex { position: (1.0, -1.0, -1.0).into(), color: (1.0, 0.0, 0.0).into() }, // ruf 4
        Vertex { position: (1.0, -1.0, 1.0).into(), color: (1.0, 0.0, 1.0).into() }, // rub 5
        Vertex { position: (1.0, 1.0, -1.0).into(), color: (1.0, 1.0, 0.0).into() }, // rdf 6
        Vertex { position: (1.0, 1.0, 1.0).into(), color: (1.0, 1.0, 1.0).into() }  // rdb 7
    ];
    (indices, vertices)
}