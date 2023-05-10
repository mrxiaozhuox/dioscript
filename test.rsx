@name = "hello rsx";
@dark = false;

@root_class = "bg-gray-800";

@sub_components = div { "Hello world" };

@data = ["u1", "u2", "u3"];

if "@dark" {
	return "dark theme unsupport.";
}

return div {
	class: @root_class,
	h1 { "title" },
	h2 { name: "hello" },
	div {
		class: "prose",
		img {
			src: "avatar.jpg"
		},
		span { "YuKun Liu" },
		@sub_components,
	}
};
